//! Profiling Playbook - GPU profiling methodology and tools

use clap::Parser;
use profiling_playbook::{run_lab, Args};
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
    println!("Profiling Playbook Results:");
    println!("  Tool: {}", result.tool);
    println!("  Wall Time: {:.2} ms", result.wall_time_ms);
    println!("  RPS: {:.2}", result.rps);
    println!("  Methodology Score: {:.2}", result.methodology_score);
    println!("  Roofline Score: {:.2}", result.roofline_score);
    println!("  Perturbation Score: {:.2}", result.perturbation_score);
    println!("  Comparability Score: {:.2}", result.comparability_score);
    println!("  Recommendation: {}", result.tool_recommendation);
    
    Ok(())
}