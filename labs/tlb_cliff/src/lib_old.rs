//! TLB Cliff Lab - Demonstrates memory access patterns and TLB behavior

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use clap::Parser;
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};

/// Command line arguments for the TLB Cliff Lab
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Access pattern: pointer-chasing or contiguous
    #[arg(long, default_value = "pointer-chasing")]
    pub access_pattern: String,

    /// Number of requests to process
    #[arg(long, default_value_t = 2000)]
    pub requests: usize,

    /// Number of warmup requests to process
    #[arg(long, default_value_t = 1000)]
    pub warmup_requests: usize,

    /// Array size for memory allocation
    #[arg(long, default_value_t = 100000)]
    pub array_size: usize,

    /// Output JSON file path
    #[arg(long)]
    pub out: Option<String>,
}

/// Result structure for the TLB Cliff Lab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlbCliffResult {
    pub access_pattern: String,
    pub requests: usize,
    pub warmup_requests: usize,
    pub array_size: usize,
    pub wall_time_ms: f64,
    pub rps: f64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
    pub tlb_misses: u64,
    pub page_walks: u64,
    pub timestamp: u64,
}

/// Memory access pattern for the lab
pub enum AccessPattern {
    PointerChasing,
    Contiguous,
}

/// Shared state for the TLB Cliff Lab
pub struct TlbState {
    pub completed: AtomicU64,
    pub histogram: Arc<Mutex<Histogram<u64>>>,
    pub tlb_misses: AtomicU64,
    pub page_walks: AtomicU64,
}

impl TlbState {
    pub fn new() -> Self {
        Self {
            completed: AtomicU64::new(0),
            histogram: Arc::new(Mutex::new(Histogram::new(3).unwrap())),
            tlb_misses: AtomicU64::new(0),
            page_walks: AtomicU64::new(0),
        }
    }
}

/// Run the TLB Cliff Lab
pub async fn run_lab(args: Args) -> Result<TlbCliffResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Initialize state
    let state = Arc::new(TlbState::new());
    
    // Create memory arrays
    let array_size = args.array_size;
    let mut pointer_array = vec![0usize; array_size];
    let data_array = vec![0u64; array_size];
    
    // Initialize pointer array for pointer-chasing
    if args.access_pattern == "pointer-chasing" {
        for i in 0..array_size - 1 {
            pointer_array[i] = i + 1;
        }
        pointer_array[array_size - 1] = 0; // Loop back to start
    }
    
    // Spawn worker threads
    let num_threads = num_cpus::get();
    let mut handles = Vec::new();
    
    for _ in 0..num_threads {
        let state = state.clone();
        let args = args.clone();
        let pointer_array = pointer_array.clone();
        let data_array = data_array.clone();
        
        let handle = tokio::spawn(async move {
            let mut total_time = 0u64;
            let mut accesses = 0u64;
            
            // Determine access pattern
            let pattern = if args.access_pattern == "pointer-chasing" {
                AccessPattern::PointerChasing
            } else {
                AccessPattern::Contiguous
            };
            
            // Process requests
            for i in 0..(args.requests / num_threads + 1) {
                let start_time = Instant::now();
                
                match pattern {
                    AccessPattern::PointerChasing => {
                        // Pointer chasing pattern
                        let mut current_index = 0;
                        for _ in 0..100 { // Access 100 elements
                            current_index = pointer_array[current_index];
                            let _ = data_array[current_index];
                        }
                    }
                    AccessPattern::Contiguous => {
                        // Contiguous access pattern
                        let base_index = (i * 100) % array_size;
                        for j in 0..100 {
                            let index = (base_index + j) % array_size;
                            let _ = data_array[index];
                        }
                    }
                }
                
                let elapsed = start_time.elapsed();
                total_time += elapsed.as_micros() as u64;
                accesses += 1;
                
                // Update counters
                state.completed.fetch_add(1, Ordering::Relaxed);
            }
            
            // Record timing
            {
                let mut histogram = state.histogram.lock().unwrap();
                histogram.record(total_time / accesses).unwrap();
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
    
    let histogram = state.histogram.lock().unwrap();
    let p50 = histogram.value_at_percentile(50.0);
    let p95 = histogram.value_at_percentile(95.0);
    let p99 = histogram.value_at_percentile(99.0);
    let max = histogram.max();
    
    // For this simplified implementation, we'll set these to 0
    // In a real implementation, we would capture actual TLB/perf counters
    let tlb_misses = 0;
    let page_walks = 0;
    
    let result = TlbCliffResult {
        access_pattern: args.access_pattern,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        array_size: args.array_size,
        wall_time_ms: wall_time,
        rps,
        p50_us: p50,
        p95_us: p95,
        p99_us: p99,
        max_us: max,
        tlb_misses,
        page_walks,
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
    async fn test_memory_access_patterns() {
        // Test that memory access patterns work
        let args = Args {
            access_pattern: "contiguous".to_string(),
            requests: 10,
            warmup_requests: 1,
            array_size: 1000,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
        assert!(result.p50_us > 0);
    }
}