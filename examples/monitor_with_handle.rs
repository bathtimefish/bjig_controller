//! Monitor command with handle (external control) example
//!
//! This example demonstrates how to start a monitor with a handle
//! that allows external control. The monitor can be stopped gracefully
//! from external code at any time.
//!
//! # How it works
//!
//! 1. Start monitor with `start_with_handle()` which returns a `MonitorHandle`
//! 2. Monitor runs in a background task
//! 3. External code can stop monitor by calling `handle.stop().await`
//! 4. Handle automatically stops monitor when dropped
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example (will monitor for 10 seconds then stop)
//! cargo run --example monitor_with_handle
//! ```
//!
//! # Expected Output
//!
//! You will see JSON uplink data displayed in real-time for 10 seconds:
//!
//! ```text
//! === BraveJIG Monitor with Handle Example ===
//!
//! Starting monitor with handle...
//! Monitor started, will run for 10 seconds...
//!
//! {"sensor_id":"0121",...}
//! {"sensor_id":"0121",...}
//! {"sensor_id":"0121",...}
//!
//! Stopping monitor...
//! Monitor stopped successfully
//!
//! === Monitor with handle example completed ===
//! ```

use bjig_controller::BjigController;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Monitor with Handle Example ===\n");

    // Create controller from environment variables
    let bjig = BjigController::from_env()?;

    println!("Starting monitor with handle...");

    // Start monitor with handle for external control
    let handle = bjig.monitor().start_with_handle().await?;

    println!("Monitor started, will run for 10 seconds...\n");

    // Let monitor run for 10 seconds
    sleep(Duration::from_secs(10)).await;

    // Stop monitor gracefully
    println!("\nStopping monitor...");
    handle.stop().await?;

    println!("Monitor stopped successfully");

    println!("\n=== Monitor with handle example completed ===");

    Ok(())
}
