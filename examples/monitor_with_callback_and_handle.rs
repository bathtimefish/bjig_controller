//! Monitor command with callback and handle example
//!
//! This example demonstrates how to combine callback functionality with
//! external control via handle. The callback processes each JSON line,
//! and the handle allows stopping the monitor from external code.
//!
//! # How it works
//!
//! 1. Start monitor with `start_with_callback_and_handle()`
//! 2. Callback processes each received JSON line
//! 3. Counter tracks number of received messages
//! 4. External code can stop monitor via handle at any time
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example
//! cargo run --example monitor_with_callback_and_handle
//! ```
//!
//! # Expected Output
//!
//! ```text
//! === BraveJIG Monitor with Callback and Handle Example ===
//!
//! Starting monitor with callback and handle...
//! Monitor started, will stop after 5 messages or 30 seconds...
//!
//! [1] Received: {"sensor_id":"0121",...}
//! [2] Received: {"sensor_id":"0121",...}
//! [3] Received: {"sensor_id":"0121",...}
//! [4] Received: {"sensor_id":"0121",...}
//! [5] Received: {"sensor_id":"0121",...}
//!
//! Received 5 messages, stopping monitor...
//! Monitor stopped successfully
//!
//! Total messages received: 5
//!
//! === Example completed ===
//! ```

use bjig_controller::BjigController;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, timeout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Monitor with Callback and Handle Example ===\n");

    // Create controller from environment variables
    let bjig = BjigController::from_env()?;

    println!("Starting monitor with callback and handle...");

    // Shared counter for messages
    let message_count = Arc::new(Mutex::new(0));
    let count_clone = message_count.clone();

    // Start monitor with callback and handle
    let handle = bjig
        .monitor()
        .start_with_callback_and_handle(move |line| {
            let mut count = count_clone.lock().unwrap();
            *count += 1;
            println!("[{}] Received: {}", *count, line);

            // Continue receiving (external code will stop via handle)
            Ok(true)
        })
        .await?;

    println!("Monitor started, will stop after 5 messages or 30 seconds...\n");

    // Poll until we receive 5 messages or 30 seconds timeout
    let result = timeout(Duration::from_secs(30), async {
        loop {
            sleep(Duration::from_millis(100)).await;
            let count = *message_count.lock().unwrap();
            if count >= 5 {
                break;
            }
        }
    })
    .await;

    let final_count = *message_count.lock().unwrap();

    match result {
        Ok(_) => {
            println!(
                "\nReceived {} messages, stopping monitor...",
                final_count
            );
        }
        Err(_) => {
            println!("\n30 second timeout reached, stopping monitor...");
        }
    }

    // Stop monitor
    handle.stop().await?;

    println!("Monitor stopped successfully");

    println!("\nTotal messages received: {}", final_count);

    println!("\n=== Example completed ===");

    Ok(())
}
