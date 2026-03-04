//! GPU Guardrail - Enforces GPU memory headroom policies

use clap::Parser;
use gpu_guardrail::{run_lab, Args};
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
    println!("GPU Guardrail Results:");
    println!("  GPU ID: {}", result.gpu_id);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  Memory Used: {} MB", result.memory_used / (1024 * 1024));
    println!("  Memory Total: {} MB", result.memory_total / (1024 * 1024));
    println!("  Memory Free: {} MB", result.memory_free / (1024 * 1024));
    println!("  Headroom Status: {}", result.headroom_status);
    println!("  Throttled Requests: {}", result.throttle_count);
    println!("  Denied Requests: {}", result.deny_count);
    
    Ok(())
}