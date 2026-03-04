//! CFS Cliff Lab - Demonstrates CPU scheduler effects under contention
//! This module implements the core logic for the CFS Cliff Lab

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use clap::Parser;
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};

/// Command line arguments for the CFS Cliff Lab
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Mode: contended or sharded
    #[arg(long, default_value = "contended")]
    pub mode: String,

    /// Number of worker threads (0 means 4 * cpu_count)
    #[arg(long, default_value_t = 0)]
    pub workers: usize,

    /// Number of producer threads (0 means cpu_count)
    #[arg(long, default_value_t = 0)]
    pub producers: usize,

    /// Number of requests to process
    #[arg(long, default_value_t = 2000)]
    pub requests: usize,

    /// Number of warmup requests to process
    #[arg(long, default_value_t = 1000)]
    pub warmup_requests: usize,

    /// Queue capacity
    #[arg(long, default_value_t = 4096)]
    pub queue: usize,

    /// Busy spin microseconds outside lock
    #[arg(long, default_value_t = 100)]
    pub work_us: u64,

    /// Busy spin microseconds inside lock
    #[arg(long, default_value_t = 1000)]
    pub lock_hold_us: u64,

    /// Number of shards for sharded mode
    #[arg(long, default_value_t = 64)]
    pub shards: usize,

    /// Output JSON file path
    #[arg(long)]
    pub out: Option<String>,
}

/// Result structure for the CFS Cliff Lab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfsCliffResult {
    pub mode: String,
    pub workers: usize,
    pub producers: usize,
    pub requests: usize,
    pub warmup_requests: usize,
    pub queue: usize,
    pub work_us: u64,
    pub lock_hold_us: u64,
    pub shards: usize,
    pub wall_time_ms: f64,
    pub rps: f64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub max_us: u64,
    pub status_counts: StatusCounts,
    pub timestamp: u64,
}

/// Status counts for the result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCounts {
    pub success: u64,
    pub timeout: u64,
    pub error: u64,
}

/// Task structure for the lab
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub timestamp: u64,
}

/// Shared state for the lab
pub struct LabState {
    pub queue: Arc<tokio::sync::Mutex<Vec<Task>>>,
    pub global_mutex: Arc<Mutex<u64>>,
    pub shards: Vec<Arc<Mutex<u64>>>,
    pub completed: AtomicU64,
    pub success_count: AtomicU64,
    pub timeout_count: AtomicU64,
    pub error_count: AtomicU64,
    pub histogram: Arc<Mutex<Histogram<u64>>>,
}

impl LabState {
    pub fn new(shards: usize) -> Self {
        let shards = (0..shards)
            .map(|_| Arc::new(Mutex::new(0u64)))
            .collect::<Vec<_>>();
        
        Self {
            queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            global_mutex: Arc::new(Mutex::new(0u64)),
            shards,
            completed: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            timeout_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            histogram: Arc::new(Mutex::new(Histogram::new(3).unwrap())),
        }
    }
}

/// Busy spin for specified microseconds
pub fn busy_spin(us: u64) {
    let start = Instant::now();
    let target = Duration::from_micros(us);
    
    while start.elapsed() < target {
        // Busy spin
    }
}

/// Run the CFS Cliff Lab
pub async fn run_lab(args: Args) -> Result<CfsCliffResult, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    // Initialize state
    let state = Arc::new(LabState::new(args.shards));
    
    // Determine thread counts
    let workers = if args.workers == 0 {
        num_cpus::get() * 4
    } else {
        args.workers
    };
    
    let producers = if args.producers == 0 {
        num_cpus::get()
    } else {
        args.producers
    };
    
    // Spawn producers
    let mut handles = Vec::new();
    for i in 0..producers {
        let queue = state.queue.clone();
        let task_id = i as u64;
        let handle = tokio::spawn(async move {
            let mut task_id = task_id;
            for _ in 0..(args.requests / producers + 1) {
                task_id += producers as u64;
                let task = Task {
                    id: task_id,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64,
                };
                
                // Try to add to queue with timeout
                let mut queue_guard = queue.lock().await;
                if queue_guard.len() < args.queue {
                    queue_guard.push(task);
                }
                drop(queue_guard);
                
                // Busy spin outside lock
                busy_spin(args.work_us);
            }
        });
        handles.push(handle);
    }
    
    // Spawn workers
    for _ in 0..workers {
        let state = state.clone();
        let args = args.clone();
        let handle = tokio::spawn(async move {
            loop {
                // Try to get a task from queue
                let task = {
                    let mut queue_guard = state.queue.lock().await;
                    if queue_guard.is_empty() {
                        drop(queue_guard);
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        continue;
                    }
                    queue_guard.pop()
                };
                
                if let Some(task) = task {
                    // Process task
                    let start_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64;
                    
                    // Busy spin outside lock
                    busy_spin(args.work_us);
                    
                    // Critical section - lock contention
                    let lock_result: Result<(), Box<dyn std::error::Error>> = if args.mode == "sharded" {
                        let shard_index = (task.id % args.shards as u64) as usize;
                        let _lock = state.shards[shard_index].lock().unwrap();
                        busy_spin(args.lock_hold_us);
                        Ok(())
                    } else {
                        let _lock = state.global_mutex.lock().unwrap();
                        busy_spin(args.lock_hold_us);
                        Ok(())
                    };
                    
                    let end_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros() as u64;
                    
                    // Record timing
                    let latency = end_time - start_time;
                    {
                        let mut histogram = state.histogram.lock().unwrap();
                        histogram.record(latency).unwrap();
                    }
                    
                    // Update counters
                    if lock_result.is_ok() {
                        state.success_count.fetch_add(1, Ordering::Relaxed);
                    } else {
                        state.error_count.fetch_add(1, Ordering::Relaxed);
                    }
                    
                    state.completed.fetch_add(1, Ordering::Relaxed);
                } else {
                    // No tasks, sleep briefly
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                
                // Check if we've completed all requests
                if state.completed.load(Ordering::Relaxed) >= args.requests as u64 {
                    break;
                }
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
    
    let result = CfsCliffResult {
        mode: args.mode,
        workers,
        producers,
        requests: args.requests,
        warmup_requests: args.warmup_requests,
        queue: args.queue,
        work_us: args.work_us,
        lock_hold_us: args.lock_hold_us,
        shards: args.shards,
        wall_time_ms: wall_time,
        rps,
        p50_us: p50,
        p95_us: p95,
        p99_us: p99,
        max_us: max,
        status_counts: StatusCounts {
            success: state.success_count.load(Ordering::Relaxed),
            timeout: state.timeout_count.load(Ordering::Relaxed),
            error: state.error_count.load(Ordering::Relaxed),
        },
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
    async fn test_busy_spin() {
        // Test that busy_spin returns quickly when 0
        busy_spin(0);
        // Should complete without blocking
    }

    #[tokio::test]
    async fn test_sharding_logic() {
        // Test that sharding logic is stable
        let shards = 64;
        let id = 12345u64;
        let shard_index = (id % shards as u64) as usize;
        assert!(shard_index < shards);
    }

    #[tokio::test]
    async fn test_json_serialization() {
        // Test that result can be serialized/deserialized
        let result = CfsCliffResult {
            mode: "contended".to_string(),
            workers: 4,
            producers: 2,
            requests: 1000,
            warmup_requests: 100,
            queue: 1024,
            work_us: 100,
            lock_hold_us: 1000,
            shards: 64,
            wall_time_ms: 100.0,
            rps: 10000.0,
            p50_us: 1000,
            p95_us: 2000,
            p99_us: 3000,
            max_us: 5000,
            status_counts: StatusCounts {
                success: 950,
                timeout: 25,
                error: 25,
            },
            timestamp: 1234567890,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let parsed: CfsCliffResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.mode, parsed.mode);
        assert_eq!(result.p99_us, parsed.p99_us);
    }

    #[tokio::test]
    async fn test_smoke_simulation() {
        // Smoke test with small parameters
        let args = Args {
            mode: "contended".to_string(),
            workers: 2,
            producers: 1,
            requests: 10,
            warmup_requests: 1,
            queue: 1024,
            work_us: 1,
            lock_hold_us: 1,
            shards: 4,
            out: None,
        };
        
        let result = run_lab(args).await.unwrap();
        
        // Verify basic properties
        assert!(result.wall_time_ms > 0.0);
        assert!(result.rps > 0.0);
        assert!(result.p50_us > 0);
        assert!(result.p95_us > 0);
        assert!(result.p99_us > 0);
        assert!(result.max_us > 0);
        assert!(result.status_counts.success > 0);
    }
}