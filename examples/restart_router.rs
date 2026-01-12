//! Router restart example
//!
//! This example demonstrates router restart sequence:
//! 1. Stop router
//! 2. Wait 10 seconds
//! 3. Start router
//! 4. Wait 1 second
//! 5. Verify with get-version
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example
//! cargo run --example restart_router
//! ```

use bjig_controller::BjigController;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Router Restart Example ===\n");

    // Create controller from environment variables
    let bjig = BjigController::from_env()?;

    // Step 1: Stop router
    println!("Step 1: Stopping router...");
    match bjig.router().stop().await {
        Ok(result) => {
            if result.is_success() {
                println!("✓ Router stopped successfully\n");
            } else {
                println!("⚠ Router stop failed: {}\n", result.message);
            }
        }
        Err(e) => {
            println!("✗ Failed to stop router: {}\n", e);
            return Err(e.into());
        }
    }

    // Step 2: Wait 10 seconds
    println!("Step 2: Waiting 10 seconds...");
    for i in (1..=10).rev() {
        print!("\r  {} seconds remaining...", i);
        use std::io::Write;
        std::io::stdout().flush()?;
        sleep(Duration::from_secs(1)).await;
    }
    println!("\r✓ Wait completed              \n");

    // Step 3: Start router
    println!("Step 3: Starting router...");
    match bjig.router().start().await {
        Ok(result) => {
            if result.is_success() {
                println!("✓ Router started successfully\n");
            } else {
                println!("⚠ Router start failed: {}\n", result.message);
                return Ok(());
            }
        }
        Err(e) => {
            println!("✗ Failed to start router: {}\n", e);
            return Err(e.into());
        }
    }

    // Step 4: Wait 1 second
    println!("Step 4: Waiting 1 second for router initialization...");
    sleep(Duration::from_secs(1)).await;
    println!("✓ Wait completed\n");

    // Step 5: Verify with get-version
    println!("Step 5: Verifying router status with get-version...");
    match bjig.router().get_version().await {
        Ok(version) => {
            println!("✓ Router is operational!");
            println!("  Version: {}.{}.{}", version.major, version.minor, version.build);
        }
        Err(e) => {
            println!("✗ Failed to get version: {}", e);
            println!("  Router may need more time to initialize");
            return Err(e.into());
        }
    }

    println!("\n=== Router restart completed successfully ===");

    Ok(())
}
