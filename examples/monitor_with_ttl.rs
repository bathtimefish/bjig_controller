//! Monitor command with TTL (timeout) and callback example
//!
//! This example demonstrates the monitor command with a timeout that
//! collects uplink data from the router in JSON format. Each JSON line is:
//! 1. Printed to stdout immediately as it arrives
//! 2. Collected in a list
//! 3. When 5 items are collected OR 120 seconds timeout, the list is printed
//!    and monitoring stops
//!
//! # How it works
//!
//! 1. The `bjig monitor --ttl 120` process is spawned with stdout piped
//! 2. Output is read line-by-line using async buffered I/O
//! 3. For each JSON line:
//!    - Print it to stdout
//!    - Add it to a collection list
//!    - If list reaches 5 items, stop monitoring
//! 4. If 120 seconds timeout is reached first, stop automatically
//! 5. Print the collected list and terminate the monitor process
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example (stops after 5 items OR 120 seconds, whichever comes first)
//! cargo run --example monitor_with_ttl
//! ```
//!
//! # Expected Output
//!
//! You will see JSON uplink data displayed in real-time, and either
//! after 5 items or 120 seconds timeout, the collected list will be displayed:
//!
//! ```text
//! Received JSON #1: {"sensor_id":"0121",...}
//! Received JSON #2: {"sensor_id":"0121",...}
//! Received JSON #3: {"sensor_id":"0121",...}
//! Received JSON #4: {"sensor_id":"0121",...}
//! Received JSON #5: {"sensor_id":"0121",...}
//!
//! Collected 5 JSON items (stopped by reaching limit):
//! [
//!   {"sensor_id":"0121",...},
//!   {"sensor_id":"0121",...},
//!   ...
//! ]
//! ```
//!
//! Or if timeout is reached first:
//!
//! ```text
//! Received JSON #1: {"sensor_id":"0121",...}
//! Received JSON #2: {"sensor_id":"0121",...}
//!
//! Collected 2 JSON items (stopped by 120 second timeout):
//! [
//!   {"sensor_id":"0121",...},
//!   {"sensor_id":"0121",...}
//! ]
//! ```

use bjig_controller::BjigController;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Monitor with TTL Example (Collect up to 5 items) ===\n");

    // Create controller from environment variables
    let bjig = BjigController::from_env()?;

    let ttl_seconds = 120;
    println!("Starting real-time monitor with {} second timeout...", ttl_seconds);
    println!("Collecting JSON uplink data until 5 items OR timeout is reached.\n");
    println!("--- Monitor Output (streaming) ---\n");

    // List to collect JSON data
    let mut json_list: Vec<String> = Vec::new();

    // Start monitoring with TTL and callback
    match bjig
        .monitor()
        .start_with_ttl_and_callback(ttl_seconds, |line| {
            // 1. Print each JSON line immediately
            println!("Received JSON #{}: {}", json_list.len() + 1, line);

            // 2. Add to list
            json_list.push(line.to_string());

            // 3. Continue until we have 5 items
            Ok(json_list.len() < 5)
        })
        .await
    {
        Ok(_) => {
            println!("\n--- End of Monitor Output ---\n");

            // Determine stop reason
            let stop_reason = if json_list.len() >= 5 {
                "stopped by reaching limit"
            } else {
                "stopped by timeout"
            };

            // Print the collected list
            println!("✓ Collected {} JSON items ({}):", json_list.len(), stop_reason);
            println!("[");
            for (i, item) in json_list.iter().enumerate() {
                if i < json_list.len() - 1 {
                    println!("  {},", item);
                } else {
                    println!("  {}", item);
                }
            }
            println!("]");
        }
        Err(e) => {
            eprintln!("\n✗ Monitor failed: {}", e);
            return Err(e.into());
        }
    }

    println!("\n=== Monitor with TTL example completed ===");

    Ok(())
}
