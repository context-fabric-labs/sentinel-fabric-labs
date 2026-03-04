//! GPU Guardrail - Enforces GPU memory headroom policies
//! This module implements the core logic for GPU memory headroom enforcement

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Command line arguments for the GPU Guardrail
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Number of requests to process (for simulation)
    #[arg(long, default_value_t = 1000)]
    pub requests: usize,

    /// Number of warmup requests to process
    #[arg(long, default_value_t = 100)]
    pub warmup_requests: usize,

    /// Minimum free memory percentage (0-100)
    #[arg(long, default_value_t = 10.0)]
    pub min_free_memory_percent: f64,

    /// GPU device ID to monitor
    #[arg(long, default_value_t = 0)]
    pub gpu_id: u32,

    /// Output JSON file path
    #[arg(long)]
    pub out: Option<String>,
}

/// Result structure for the GPU Guardrail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuGuardrailResult {
    pub gpu_id: u32,
    pub requests: usize,
    pub warmup_requests: usize,
    pub min_free_memory_percent: f64,
    pub wall_time_ms: f64,
    pub rps: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_free: u64,
    pub headroom_status: String,
    pub throttle_count: u64,
    pub deny_count: u64,
    pub timestamp: u64,
}

/// Shared state for the GPU Guardrail
pub struct GuardrailState {
    pub completed: AtomicU64,
    pub memory_used: AtomicU64,
    pub memory_total: AtomicU64,
    pub throttle_count: AtomicU64,
    pub deny_count: AtomicU64,
}

impl GuardrailState {
    pub fn new() -> Self {
        Self {
            completed: AtomicU64::new(0),
            memory_used: AtomicU64::new(0),
            memory_total: AtomicU64::new(0),
            throttle_count: AtomicU64::new(0),
            deny_count: AtomicU64::new(0),
        }
    }
}

/// Simulate GPU memory metrics
fn get_gpu_memory_metrics(gpu_id: u32) -> (u64, u64) {
    // Simulate GPU memory values (in bytes)
    let memory_total = (2000 + (gpu_id as u64 * 200)) * 1024 * 1024; // In bytes
    let memory_used = (1000 + (gpu_id as u64 * 100)) * 1024 * 1024; // In bytes
    
    (memory_used, memory_total)
}

/// Check if headroom policy is satisfied
fn check_headroom_policy(memory_used: u64, memory_total: u64, min_free_percent: f64) -> (bool, String) {
    let memory_free = memory_total.saturating_sub(memory_used);
    let free_percent = (memory_free as f64 / memory_total as f64) * 100.0;
    
    if free_percent >= min_free_percent {
        (true, "OK".to_string())
    } else {
        (false, format!("Insufficient headroom: {:.1}% free, required {:.1}%", free_percent, min_free_percent))
    }
}

/// Run the GPU Guardrail
pub async fn run_lab(args: Args) -> Result<GpuGuardrailResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Initialize state
    let state = Arc::new(GuardrailState::new());
    
    // Get GPU memory metrics
    let (memory_used, memory_total) = get_gpu_memory_metrics(args.gpu_id);
    
    // Check headroom policy
    let (policy_satisfied, headroom_status) = check_headroom_policy(
        memory_used, 
        memory_total, 
        args.min_free_memory_percent
    );
    
    // Simulate processing requests with headroom policy enforcement
    let num_threads = num_cpus::get();
    let mut handles = Vec::new();
    let total_requests = args.requests;
    
    for thread_id in 0..num_threads {
        let state = state.clone();
        let args = args.clone();
        
        let handle = tokio::spawn(async move {
            let requests_per_thread = (total_requests / num_threads) + if thread_id < (total_requests % num_threads) { 1 } else { 0 };
            
            for i in 0..requests_per_thread {
                // Simulate work that might consume GPU memory
                let start_time = Instant::now();
                
                // Simulate some GPU-like work
                let mut sum = 0u64;
                for j in 0..1000 {
                    sum = sum.wrapping_add(i as u64 * j as u64);
                }
                
                let elapsed = start_time.elapsed();
                let _time_us = elapsed.as_micros() as u64;
                
                // Simulate headroom policy enforcement
                let (policy_satisfied, _) = check_headroom_policy(
                    memory_used, 
                    memory_total, 
                    args.min_free_memory_percent
                );
                
                // Simulate throttling/denial based on policy
                if !policy_satisfied {
                    if i % 10 == 0 {
                        // Simulate throttling
                        state.throttle_count.fetch_add(1, Ordering::Relaxed);
                    } else if i % 20 == 0 {
                        // Simulate denial
                        state.deny_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
                
                state.memory_used.fetch_add(memory_used, Ordering::Relaxed);
                state.memory_total.fetch_add(memory_total, Ordering::Relaxed);
                state.completed.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.await?;
    }
    
    // Calculate results
    let wall_time = start_time.elapsed().as_millis() as f64;
    let rps = args.requests as f64 / (wall_time / 1000.0);
    
    let result = GpuGuardrailResult {
        gpu_id: args.gpu_id,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        min_free_memory_percent: args.min_free_memory_percent,
        wall_time_ms: wall_time,
        rps,
        memory_used,
        memory_total,
        memory_free: memory_total.saturating_sub(memory_used),
        headroom_status,
        throttle_count: state.throttle_count.load(Ordering::Relaxed),
        deny_count: state.deny_count.load(Ordering::Relaxed),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_headroom_policy() {
        let args = Args {
            requests: 100,
            warmup_requests: 10,
            min_free_memory_percent: 10.0,
            gpu_id: 0,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
        assert!(result.memory_used > 0);
        assert!(result.memory_total > 0);
        assert!(result.memory_free > 0);
    }
}