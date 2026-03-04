//! NUMA Cliff Lab - Demonstrates NUMA topology effects

use clap::Parser;
use numa_cliff::{run_lab, Args};
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
    println!("NUMA Cliff Lab Results:");
    println!("  Access Type: {}", result.access_type);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  P50: {} us", result.p50_us);
    println!("  P95: {} us", result.p95_us);
    println!("  P99: {} us", result.p99_us);
    println!("  Max: {} us", result.max_us);
    println!("  Remote Accesses: {}", result.remote_accesses);
    println!("  CPU Migrations: {}", result.cpu_migrations);
    
    Ok(())
}