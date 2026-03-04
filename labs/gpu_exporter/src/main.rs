//! GPU Exporter - Collects GPU metrics and exposes them via Prometheus

use clap::Parser;
use gpu_exporter::{run_lab, Args};
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
    println!("GPU Exporter Results:");
    println!("  GPU ID: {}", result.gpu_id);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  Utilization: {:.2}%", result.utilization);
    println!("  Memory Utilization: {:.2}%", result.memory_utilization);
    println!("  Temperature: {:.2}°C", result.temperature);
    println!("  Power Usage: {:.2}W", result.power_usage);
    println!("  Memory Used: {} MB", result.memory_used / (1024 * 1024));
    println!("  Memory Total: {} MB", result.memory_total / (1024 * 1024));
    
    Ok(())
}