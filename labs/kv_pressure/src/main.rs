//! KV Pressure - Estimates KV/cache pressure from sequence length and concurrency

use clap::Parser;
use kv_pressure::{run_lab, Args};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Run the lab
    let result = run_lab(args.clone()).await?;
    
    // Write output if specified
    if let Some(out_path) = &args.out {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(out_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write JSON result
        let json = serde_json::to_string_pretty(&result)?;
        fs::write(out_path, json)?;
        println!("Results written to: {}", out_path);
    }
    
    // Print summary
    println!("KV Pressure Results:");
    println!("  GPU ID: {}", result.gpu_id);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  Avg Sequence Length: {}", result.avg_seq_len);
    println!("  Concurrency: {}", result.concurrency);
    println!("  Pressure Score: {:.2}", result.pressure_score);
    println!("  Memory Estimate: {} KB", result.memory_estimate / 1024);
    println!("  Cache Hit Rate: {:.2}%", result.cache_hit_rate);
    println!("  Pressure Level: {}", result.pressure_level);
    
    Ok(())
}