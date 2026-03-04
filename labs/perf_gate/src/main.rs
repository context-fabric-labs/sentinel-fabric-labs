//! Perf Stat Gate - Performance regression testing framework

use clap::Parser;
use perf_gate::{run_lab, Args};
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
    println!("Perf Stat Gate Results:");
    println!("  Run Type: {}", result.run_type);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  Context Switches: {}", result.context_switches);
    println!("  CPU Migrations: {}", result.cpu_migrations);
    println!("  Task Clock: {}", result.task_clock);
    println!("  Cycles: {}", result.cycles);
    println!("  Instructions: {}", result.instructions);
    
    Ok(())
}