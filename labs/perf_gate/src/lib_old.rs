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
    pub completed: AtomicU64,
    pub total_time: AtomicU64,
    pub context_switches: AtomicU64,
    pub cpu_migrations: AtomicU64,
    pub task_clock: AtomicU64,
    pub cycles: AtomicU64,
    pub instructions: AtomicU64,
}

impl PerfState {
    pub fn new() -> Self {
        Self {
            completed: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
            cpu_migrations: AtomicU64::new(0),
            task_clock: AtomicU64::new(0),
            cycles: AtomicU64::new(0),
            instructions: AtomicU64::new(0),
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
    
    for _ in 0..num_threads {
        let state = state.clone();
        let args = args.clone();
        
        let handle = tokio::spawn(async move {
            // Process requests
            for i in 0..(args.requests / num_threads + 1) {
                // Simulate some work
                let start_time = Instant::now();
                
                // Do some CPU-bound work to simulate real workload
                let mut _sum = 0u64;
                for j in 0..1000 {
                    _sum += (i as u64) * (j as u64);
                }
                
                let elapsed = start_time.elapsed();
                let time_us = elapsed.as_micros() as u64;
                
                // Update counters
                state.completed.fetch_add(1, Ordering::Relaxed);
                state.total_time.fetch_add(time_us, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }
    
    // Calculate results
    let wall_time = start_time.elapsed().as_millis() as f64;
    let rps = args.requests as f64 / (wall_time / 1000.0);
    
    // For this simplified implementation, we'll set perf counters to 0
    // In a real implementation, we would capture actual perf stats
    let context_switches = 0;
    let cpu_migrations = 0;
    let task_clock = 0;
    let cycles = 0;
    let instructions = 0;
    
    let result = PerfGateResult {
        run_type: args.run_type,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        wall_time_ms: wall_time,
        rps,
        context_switches,
        cpu_migrations,
        task_clock,
        cycles,
        instructions,
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
    async fn test_perf_gate_run() {
        // Test that perf gate run works
        let args = Args {
            run_type: "bad".to_string(),
            requests: 1000,
            warmup_requests: 100,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
    }
}