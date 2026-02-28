//! S1.6 – Upstream health probe + circuit breaker (CLOSED/OPEN/HALF_OPEN)
//!
//! - Background task probes /health every N seconds
//! - Tracks consecutive failures and last ok
//! - Exposes state; main handler checks and may fast-fail
//! - Allows a small number of trial requests in HALF_OPEN

use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

pub struct BreakerConfig {
    pub failure_threshold: u32, // open after N consecutive failures
    pub half_open_probe_interval: Duration, // how often to allow probes in half-open
    pub health_interval: Duration, // background probe interval
    pub half_open_max_inflight: usize, // limited concurrency allowed in half-open
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            half_open_probe_interval: Duration::from_secs(2),
            health_interval: Duration::from_secs(2),
            half_open_max_inflight: 2,
        }
    }
}

pub struct Breaker {
    state: AtomicU8,
    failures: AtomicU32,
    last_ok_epoch_ms: AtomicI64,
    half_open_slots: Arc<Semaphore>,
}

impl Breaker {
    pub fn new(cfg: &BreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(State::Closed as u8),
            failures: AtomicU32::new(0),
            last_ok_epoch_ms: AtomicI64::new(now_ms()),
            half_open_slots: Arc::new(Semaphore::new(cfg.half_open_max_inflight)),
        }
    }

    pub fn state(&self) -> State {
        match self.state.load(Ordering::Relaxed) {
            0 => State::Closed,
            1 => State::Open,
            _ => State::HalfOpen,
        }
    }

    pub fn on_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        self.state.store(State::Closed as u8, Ordering::Relaxed);
        self.last_ok_epoch_ms.store(now_ms(), Ordering::Relaxed);
        metrics::gauge!("sentinel_breaker_state").set(State::Closed as u8 as f64);
    }

    pub fn on_failure(&self, cfg: &BreakerConfig) {
        let f = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        if f >= cfg.failure_threshold {
            let prev = self.state.swap(State::Open as u8, Ordering::Relaxed);
            if prev != State::Open as u8 {
                metrics::counter!("sentinel_breaker_transitions_total", "from" => fmt_state(prev), "to" => fmt_state(State::Open as u8)).increment(1);
            }
            metrics::gauge!("sentinel_breaker_state").set(State::Open as u8 as f64);
        }
    }

    /// Check if request may proceed.
    /// - CLOSED: allowed, returns None (no special guard)
    /// - OPEN: denied, returns Err(())
    /// - HALF_OPEN: allowed if slot available, returns Some(HalfOpenGuard)
    pub async fn allow(&self, _cfg: &BreakerConfig) -> Result<Option<HalfOpenGuard>, ()> {
        match self.state() {
            State::Closed => Ok(None),
            State::Open => Err(()),
            State::HalfOpen => match self.half_open_slots.clone().acquire_owned().await {
                Ok(permit) => Ok(Some(HalfOpenGuard { _permit: permit })),
                Err(_) => Err(()),
            },
        }
    }

    pub fn move_to_half_open(&self) {
        let prev = self.state.swap(State::HalfOpen as u8, Ordering::Relaxed);
        if prev != State::HalfOpen as u8 {
            metrics::counter!("sentinel_breaker_transitions_total", "from" => fmt_state(prev), "to" => fmt_state(State::HalfOpen as u8)).increment(1);
        }
        metrics::gauge!("sentinel_breaker_state").set(State::HalfOpen as u8 as f64);
    }

    pub fn open(&self) {
        let prev = self.state.swap(State::Open as u8, Ordering::Relaxed);
        if prev != State::Open as u8 {
            metrics::counter!("sentinel_breaker_transitions_total", "from" => fmt_state(prev), "to" => fmt_state(State::Open as u8)).increment(1);
        }
        metrics::gauge!("sentinel_breaker_state").set(State::Open as u8 as f64);
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
fn fmt_state(u: u8) -> &'static str {
    match u {
        0 => "CLOSED",
        1 => "OPEN",
        _ => "HALF_OPEN",
    }
}

/// Guard that holds a HALF_OPEN slot permit until dropped.
pub struct HalfOpenGuard {
    _permit: OwnedSemaphorePermit,
}

/// Background health probe task. On HTTP 200, `on_success`. On error, `on_failure`.
pub async fn run_health_probe(breaker: Arc<Breaker>, cfg: BreakerConfig, upstream_base: String) {
    let client = reqwest::Client::new();
    loop {
        let url = format!("{}/health", upstream_base);
        let ok = match client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        };
        if ok {
            breaker.on_success();
        } else {
            breaker.on_failure(&cfg);
        }

        // If OPEN, after one interval, move to HALF_OPEN to allow probes
        if breaker.state() == State::Open {
            tokio::time::sleep(cfg.half_open_probe_interval).await;
            breaker.move_to_half_open();
        }
        tokio::time::sleep(cfg.health_interval).await;
    }
}
