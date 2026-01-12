//! Module control example
//!
//! This example demonstrates module operations:
//! - Instant uplink (get sensor data)
//! - Get/set parameters
//!
//! # Usage
//!
//! ```bash
//! # Set environment variables
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//!
//! # Run example (replace with your module ID)
//! cargo run --example module_control
//! ```

use bjig_controller::BjigController;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== BraveJIG Module Control Example ===\n");

    // Create controller
    let bjig = BjigController::from_env()?;

    // Example sensor IDs and module IDs
    // Replace these with your actual values
    let sensor_id = "0121"; // Illuminance sensor
    let module_id = "2468800203400004"; // Example module ID

    println!("Sensor ID: {}", sensor_id);
    println!("Module ID: {}\n", module_id);

    // Get instant uplink (sensor data)
    println!("Requesting instant uplink...");
    match bjig.module(sensor_id, module_id).instant_uplink().await {
        Ok(data) => {
            println!("✓ Sensor data received:");
            println!("{}\n", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("✗ Failed to get sensor data: {}\n", e);
        }
    }

    // Get module parameters
    println!("Getting module parameters...");
    match bjig.module(sensor_id, module_id).get_parameter().await {
        Ok(params) => {
            println!("✓ Module parameters:");
            println!("{}\n", serde_json::to_string_pretty(&params)?);
        }
        Err(e) => {
            println!("✗ Failed to get parameters: {}\n", e);
        }
    }

    println!("=== Example completed ===");

    Ok(())
}
