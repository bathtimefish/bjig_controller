//! Simple router control example
//!
//! This example demonstrates basic router operations:
//! - Get firmware version
//! - Start/stop router
//! - Get module IDs
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example
//! cargo run --example simple_router
//! ```

use bjig_controller::BjigController;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Router Control Example ===\n");

    // Create controller from environment variables
    let bjig = BjigController::from_env()?;

    // Get router version
    println!("Getting router version...");
    let version = bjig.router().get_version().await?;
    println!("✓ Router version: {}.{}.{}\n", version.major, version.minor, version.build);

    // Get scan mode
    println!("Getting scan mode...");
    let mode = bjig.router().get_scan_mode().await?;
    println!("✓ Scan mode: {} ({})\n", mode.mode, mode.mode_name);

    // Get module IDs
    println!("Getting module IDs...");
    let modules = bjig.router().get_module_id(None).await?;
    println!("✓ Found {} module(s):", modules.module_count);
    for (i, module_id) in modules.modules.iter().enumerate() {
        println!("  [{}] {}", i, module_id);
    }

    println!("\n=== Example completed successfully ===");

    Ok(())
}
