//! Environment variable configuration example
//!
//! This example demonstrates how to use environment variables
//! to configure the controller.
//!
//! # Usage
//!
//! ```bash
//! # Set all environment variables
//! export BJIG_CLI_BIN_PATH=./bin/bjig
//! export BJIG_CLI_PORT=/dev/ttyACM0
//! export BJIG_CLI_BAUD=115200
//! export BJIG_CLI_MODULE_CONFIG=./module-config.yml
//!
//! # Run example
//! cargo run --example env_config
//! ```

use bjig_controller::{BjigController, ENV_BJIG_CLI_BIN_PATH, ENV_BJIG_CLI_PORT, ENV_BJIG_CLI_BAUD};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== Environment Variable Configuration Example ===\n");

    // Display current environment variables
    println!("Environment variables:");
    println!("  {} = {:?}", ENV_BJIG_CLI_BIN_PATH, env::var(ENV_BJIG_CLI_BIN_PATH).ok());
    println!("  {} = {:?}", ENV_BJIG_CLI_PORT, env::var(ENV_BJIG_CLI_PORT).ok());
    println!("  {} = {:?}", ENV_BJIG_CLI_BAUD, env::var(ENV_BJIG_CLI_BAUD).ok());
    println!();

    // Create controller from environment
    println!("Creating controller from environment variables...");
    let bjig = match BjigController::from_env() {
        Ok(b) => {
            println!("✓ Controller created successfully\n");
            b
        }
        Err(e) => {
            println!("✗ Failed to create controller: {}", e);
            println!("\nMake sure to set the required environment variables:");
            println!("  export BJIG_CLI_PORT=/dev/ttyACM0");
            println!("  export BJIG_CLI_BAUD=115200");
            return Err(e.into());
        }
    };

    // Test with a simple command
    println!("Testing with get-version command...");
    match bjig.router().get_version().await {
        Ok(version) => {
            println!("✓ Router version: {}\n", version.version);
        }
        Err(e) => {
            println!("✗ Failed: {}\n", e);
        }
    }

    // Example: Override with explicit values
    println!("You can also override environment variables:");
    println!("  let bjig = BjigController::from_env()?");
    println!("      .with_port(\"/dev/ttyACM1\")");
    println!("      .with_baud(9600);");

    println!("\n=== Example completed ===");

    Ok(())
}
