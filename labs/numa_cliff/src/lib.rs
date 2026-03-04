//! NUMA Cliff Lab - Demonstrates NUMA topology effects

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use clap::Parser;
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};

/// Command line arguments for the NUMA Cliff Lab
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Access type: local or remote
    #[arg(long, default_value = "local")]
    pub access_type: String,

    /// Number of requests to process
    #[arg(long, default_value_t = 2000)]
    pub requests: usize,

    /// Number of warmup requests to process
    #[arg(long, default_value_t = 1000)]
    pub warmup_requests: usize,

    /// Memory size for allocation
    #[arg(long, default_value_t = 100000)]
    pub memory_size: usize,

    /// Output JSON file path
    #[arg(long)]
    pub out: Option<String>,
}

/// Result structure for the NUMA Cliff Lab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumaCliffResult {
    pub access_type: String,
    pub requests: usize,
    pub warmup_requests: usize,
    pub memory_size: usize,
    pub wall_time_ms: f64,
    pub rps: f64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
    pub remote_accesses: u64,
    pub cpu_migrations: u64,
    pub timestamp: u64,
}

/// Shared state for the NUMA Cliff Lab
pub struct NumaState {
    pub histogram: Arc<Mutex<Histogram<u64>>>,
    pub remote_accesses: AtomicU64,
    pub cpu_migrations: AtomicU64,
}

impl NumaState {
    pub fn new() -> Self {
        Self {
            histogram: Arc::new(Mutex::new(Histogram::new(3).unwrap())),
            remote_accesses: AtomicU64::new(0),
            cpu_migrations: AtomicU64::new(0),
        }
    }
}

/// Run the NUMA Cliff Lab
pub async fn run_lab(args: Args) -> Result<NumaCliffResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Initialize state
    let state = Arc::new(NumaState::new());
    
    // Create memory array
    let memory_size = args.memory_size;
    let data_array = vec![0u64; memory_size];
    
    // Spawn worker threads
    let num_threads = num_cpus::get();
    let mut handles = Vec::new();
    let total_requests = args.requests;
    
    for thread_id in 0..num_threads {
        let state = state.clone();
        let args = args.clone();
        let data_array = data_array.clone();
        
        let handle = tokio::spawn(async move {
            let requests_per_thread = (total_requests / num_threads) + if thread_id < (total_requests % num_threads) { 1 } else { 0 };
            
            for _ in 0..requests_per_thread {
                let access_start = Instant::now();
                
                // Perform memory access pattern
                let _sum = if args.access_type == "local" {
                    // Local access - same NUMA node
                    let mut sum = 0u64;
                    let start_idx = (thread_id * memory_size) / num_threads;
                    let end_idx = ((thread_id + 1) * memory_size) / num_threads;
                    for i in start_idx..end_idx.min(memory_size) {
                        sum = sum.wrapping_add(data_array.get(i).copied().unwrap_or(0));
                    }
                    sum
                } else {
                    // Remote access - different NUMA node
                    let mut sum = 0u64;
                    let start_idx = ((num_threads - thread_id - 1) * memory_size) / num_threads;
                    let end_idx = ((num_threads - thread_id) * memory_size) / num_threads;
                    for i in start_idx..end_idx.min(memory_size) {
                        sum = sum.wrapping_add(data_array.get(i).copied().unwrap_or(0));
                    }
                    sum
                };
                
                let elapsed = access_start.elapsed();
                let latency = elapsed.as_micros() as u64;
                
                {
                    let mut histogram = state.histogram.lock().unwrap();
                    let _ = histogram.record(latency);
                }
                
                // Simulate NUMA effects
                if args.access_type == "remote" && latency > 100 {
                    state.remote_accesses.fetch_add(1, Ordering::Relaxed);
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
    
    let histogram = state.histogram.lock().unwrap();
    let p50 = histogram.value_at_percentile(50.0);
    let p95 = histogram.value_at_percentile(95.0);
    let p99 = histogram.value_at_percentile(99.0);
    let max = histogram.max();
    
    let result = NumaCliffResult {
        access_type: args.access_type,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        memory_size: args.memory_size,
        wall_time_ms: wall_time,
        rps,
        p50_us: p50,
        p95_us: p95,
        p99_us: p99,
        max_us: max,
        remote_accesses: state.remote_accesses.load(Ordering::Relaxed),
        cpu_migrations: 0,
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
    async fn test_numa_access_patterns() {
        let args = Args {
            access_type: "local".to_string(),
            requests: 10,
            warmup_requests: 1,
            memory_size: 1000,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
        assert!(result.p50_us > 0);
    }
}
