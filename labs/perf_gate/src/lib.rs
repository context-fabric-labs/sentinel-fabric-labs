//! Perf Stat Gate - Performance regression testing framework

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Command line arguments for the Perf Stat Gate
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Run type: bad or fixed
    #[arg(long, default_value = "bad")]
    pub run_type: String,

    /// Number of requests to process
    #[arg(long, default_value_t = 10000)]
    pub requests: usize,

    /// Number of warmup requests to process
    #[arg(long, default_value_t = 500)]
    pub warmup_requests: usize,

    /// Output JSON file path
    #[arg(long)]
    pub out: Option<String>,
}

/// Result structure for the Perf Stat Gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfGateResult {
    pub run_type: String,
    pub requests: usize,
    pub warmup_requests: usize,
    pub wall_time_ms: f64,
    pub rps: f64,
    pub context_switches: u64,
    pub cpu_migrations: u64,
    pub task_clock: u64,
    pub cycles: u64,
    pub instructions: u64,
    pub timestamp: u64,
}

/// Shared state for the Perf Stat Gate
pub struct PerfState {
    pub total_time: AtomicU64,
    pub context_switches: AtomicU64,
    pub cpu_migrations: AtomicU64,
}

impl PerfState {
    pub fn new() -> Self {
        Self {
            total_time: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            cpu_migrations: AtomicU64::new(0),
        }
    }
}

/// Run the Perf Stat Gate
pub async fn run_lab(args: Args) -> Result<PerfGateResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Initialize state
    let state = Arc::new(PerfState::new());
    
    // Spawn worker threads
    let num_threads = num_cpus::get();
    let mut handles = Vec::new();
    let total_requests = args.requests;
    
    for thread_id in 0..num_threads {
        let state = state.clone();
        let args = args.clone();
        
        let handle = tokio::spawn(async move {
            let requests_per_thread = (total_requests / num_threads) + if thread_id < (total_requests % num_threads) { 1 } else { 0 };
            
            for i in 0..requests_per_thread {
                let start_time = Instant::now();
                
                // Simulate some work
                let mut sum = 0u64;
                for j in 0..1000 {
                    sum = sum.wrapping_add((i as u64).wrapping_mul(j as u64));
                }
                
                // Simulate bad performance if run_type is "bad"
                if args.run_type == "bad" && i % 10 == 0 {
                    // Simulate some extra work for bad runs
                    for _ in 0..100 {
                        sum = sum.wrapping_add(1);
                    }
                }
                
                let elapsed = start_time.elapsed();
                let time_us = elapsed.as_micros() as u64;
                
                state.total_time.fetch_add(time_us, Ordering::Relaxed);
                
                // Simulate performance counters
                if args.run_type == "bad" && i % 20 == 0 {
                    state.context_switches.fetch_add(1, Ordering::Relaxed);
                    state.cpu_migrations.fetch_add(1, Ordering::Relaxed);
                }
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
    
    let result = PerfGateResult {
        run_type: args.run_type,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        wall_time_ms: wall_time,
        rps,
        context_switches: state.context_switches.load(Ordering::Relaxed),
        cpu_migrations: state.cpu_migrations.load(Ordering::Relaxed),
        task_clock: state.total_time.load(Ordering::Relaxed),
        cycles: (rps * 1000.0) as u64,
        instructions: args.requests as u64,
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
    async fn test_perf_gate_simulation() {
        let args = Args {
            run_type: "bad".to_string(),
            requests: 100,
            warmup_requests: 10,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
    }
}
