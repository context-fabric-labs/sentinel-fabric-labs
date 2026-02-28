//! S1.5 – Token estimation, admission (semaphore + bounded queue), degrade mode.
//!
//! Policy:
//!   CLOSED (normal):
//!     - try_acquire inflight permit  → accept
//!     - inflight full, queue permit available → queue  (wait up to QUEUE_WAIT_MS)
//!     - both full → 429 Too Many Requests
//!   DEGRADE: when queuing, clamp max_tokens in the request body to DEGRADE_MAX_TOKENS.
//!
//! Metrics emitted:
//!   sentinel_token_estimate_total       counter
//!   sentinel_token_estimate_hist        histogram  (estimated tokens)
//!   sentinel_queue_depth                gauge
//!   sentinel_admission_total{decision,reason}  counter  (accept/queue/reject/degrade)
//!   sentinel_degraded_total{reason}     counter

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError},
    time::timeout,
};

/// How long a queued request waits for an inflight slot before being rejected.
pub const QUEUE_WAIT_MS: u64 = 150;
/// When degrading, clamp max_tokens to this value.
pub const DEGRADE_MAX_TOKENS: u64 = 64;

// ── Token estimation ────────────────────────────────────────────────────────

/// Estimate tokens by summing chars in all message `content` fields, divided by 4.
/// Falls back to 0 if JSON parsing fails.
pub fn estimate_tokens(body: &[u8]) -> u64 {
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(msgs) = v.get("messages").and_then(|m| m.as_array()) {
            let chars: usize = msgs
                .iter()
                .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
                .map(|s| s.len())
                .sum();
            return (chars / 4).max(1) as u64;
        }
    }
    0
}

/// Clamp `max_tokens` in the JSON body to `ceiling`.
/// Returns the (possibly modified) body bytes.
pub fn clamp_max_tokens(body: &[u8], ceiling: u64) -> Vec<u8> {
    let mut v = match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(v) => v,
        Err(_) => return body.to_vec(),
    };

    if let Some(obj) = v.as_object_mut() {
        let current = obj.get("max_tokens").and_then(|t| t.as_u64()).unwrap_or(0);
        if current == 0 || current > ceiling {
            obj.insert(
                "max_tokens".to_string(),
                serde_json::Value::Number(ceiling.into()),
            );
        }
    }

    serde_json::to_vec(&v).unwrap_or_else(|_| body.to_vec())
}

// ── Admission state ─────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AdmissionDecision {
    Accept,
    Queue,
    Reject,
}

pub struct AdmissionConfig {
    pub max_inflight: usize,
    pub queue_slots: usize,
    pub queue_wait: Duration,
    pub degrade_max_tokens: u64,
}

impl Default for AdmissionConfig {
    fn default() -> Self {
        Self {
            max_inflight: 64,
            queue_slots: 128,
            queue_wait: Duration::from_millis(QUEUE_WAIT_MS),
            degrade_max_tokens: DEGRADE_MAX_TOKENS,
        }
    }
}

pub struct AdmissionController {
    inflight: Arc<Semaphore>,
    queue: Arc<Semaphore>,
    pub config: AdmissionConfig,
}

pub struct AdmissionGuard {
    #[allow(dead_code)]
    pub inflight_permit: OwnedSemaphorePermit,
    pub queue_permit: Option<OwnedSemaphorePermit>,
}

impl Drop for AdmissionGuard {
    fn drop(&mut self) {
        // queue permit released automatically; inflight_permit too.
        if self.queue_permit.is_some() {
            metrics::gauge!("sentinel_queue_depth").decrement(1.0);
        }
    }
}

impl AdmissionController {
    pub fn new(config: AdmissionConfig) -> Self {
        let inflight = Arc::new(Semaphore::new(config.max_inflight));
        let queue = Arc::new(Semaphore::new(config.queue_slots));
        Self {
            inflight,
            queue,
            config,
        }
    }

    /// Attempt to acquire an inflight slot.
    /// Returns Ok(guard) or Err(decision) where decision is Reject.
    pub async fn admit(&self) -> Result<AdmissionGuard, AdmissionDecision> {
        // Fast path: try direct acquire
        match self.inflight.clone().try_acquire_owned() {
            Ok(permit) => {
                metrics::counter!(
                    "sentinel_admission_total",
                    "decision" => "accept",
                    "reason" => "direct"
                )
                .increment(1);
                return Ok(AdmissionGuard {
                    inflight_permit: permit,
                    queue_permit: None,
                });
            }
            Err(TryAcquireError::NoPermits) => {} // fall through to queue
            Err(TryAcquireError::Closed) => {
                return Err(AdmissionDecision::Reject);
            }
        }

        // Slow path: try to enter queue
        let q_permit = match self.queue.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                metrics::counter!(
                    "sentinel_admission_total",
                    "decision" => "reject",
                    "reason" => "queue_full"
                )
                .increment(1);
                return Err(AdmissionDecision::Reject);
            }
        };

        metrics::gauge!("sentinel_queue_depth").increment(1.0);
        metrics::counter!(
            "sentinel_admission_total",
            "decision" => "queue",
            "reason" => "inflight_full"
        )
        .increment(1);

        // Wait for inflight slot up to queue_wait
        match timeout(
            self.config.queue_wait,
            self.inflight.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => {
                // queue permit kept alive until guard drops
                Ok(AdmissionGuard {
                    inflight_permit: permit,
                    queue_permit: Some(q_permit),
                })
            }
            Ok(Err(_)) => {
                // semaphore closed
                metrics::gauge!("sentinel_queue_depth").decrement(1.0);
                metrics::counter!(
                    "sentinel_admission_total",
                    "decision" => "reject",
                    "reason" => "sem_closed"
                )
                .increment(1);
                Err(AdmissionDecision::Reject)
            }
            Err(_elapsed) => {
                metrics::gauge!("sentinel_queue_depth").decrement(1.0);
                metrics::counter!(
                    "sentinel_admission_total",
                    "decision" => "reject",
                    "reason" => "queue_timeout"
                )
                .increment(1);
                Err(AdmissionDecision::Reject)
            }
        }
    }

    /// Whether the guard was queued (i.e. degrade should apply).
    pub fn should_degrade(guard: &AdmissionGuard) -> bool {
        guard.queue_permit.is_some()
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_basic() {
        let body = br#"{"model":"stub","messages":[{"role":"user","content":"hello world"}]}"#;
        let est = estimate_tokens(body);
        // "hello world" = 11 chars → 11/4 = 2, max(2,1) = 2
        assert_eq!(est, 2);
    }

    #[test]
    fn test_estimate_tokens_multiple_messages() {
        let body = br#"{"messages":[{"role":"user","content":"aaaa"},{"role":"assistant","content":"bbbbbbbb"}]}"#;
        // 4 + 8 = 12 chars / 4 = 3
        let est = estimate_tokens(body);
        assert_eq!(est, 3);
    }

    #[test]
    fn test_estimate_tokens_empty_body() {
        assert_eq!(estimate_tokens(b"not json"), 0);
    }

    #[test]
    fn test_estimate_tokens_no_messages() {
        let body = br#"{"model":"stub"}"#;
        assert_eq!(estimate_tokens(body), 0);
    }

    #[test]
    fn test_clamp_max_tokens_clamps() {
        let body = br#"{"model":"stub","max_tokens":512,"messages":[]}"#;
        let out = clamp_max_tokens(body, 64);
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["max_tokens"].as_u64().unwrap(), 64);
    }

    #[test]
    fn test_clamp_max_tokens_already_below() {
        let body = br#"{"model":"stub","max_tokens":16,"messages":[]}"#;
        let out = clamp_max_tokens(body, 64);
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["max_tokens"].as_u64().unwrap(), 16);
    }

    #[test]
    fn test_clamp_max_tokens_adds_if_missing() {
        let body = br#"{"model":"stub","messages":[]}"#;
        let out = clamp_max_tokens(body, 64);
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["max_tokens"].as_u64().unwrap(), 64);
    }

    #[test]
    fn test_clamp_max_tokens_invalid_json() {
        let body = b"not json";
        let out = clamp_max_tokens(body, 64);
        assert_eq!(out, body);
    }

    #[tokio::test]
    async fn test_admission_accept() {
        let ctrl = AdmissionController::new(AdmissionConfig {
            max_inflight: 2,
            queue_slots: 4,
            queue_wait: Duration::from_millis(50),
            degrade_max_tokens: 64,
        });
        let guard = ctrl.admit().await.expect("should accept");
        assert!(!AdmissionController::should_degrade(&guard));
    }

    #[tokio::test]
    async fn test_admission_reject_when_full() {
        let ctrl = AdmissionController::new(AdmissionConfig {
            max_inflight: 1,
            queue_slots: 0,
            queue_wait: Duration::from_millis(10),
            degrade_max_tokens: 64,
        });
        // Hold the only inflight slot
        let _g = ctrl.admit().await.expect("first accept");
        // Now both inflight and queue are full
        let res = ctrl.admit().await;
        assert!(res.is_err());
    }
}
