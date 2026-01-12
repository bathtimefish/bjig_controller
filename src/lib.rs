//! bjig_controller - Rust library for controlling BraveJIG USB Router
//!
//! This library provides a high-level interface to control BraveJIG USB Router
//! via the `bjig` CLI command. It wraps all bjig commands in a type-safe Rust API.
//!
//! # Features
//!
//! - **Router Control**: Start/stop router, manage firmware, configure settings
//! - **Module Management**: Control sensor modules, retrieve data, update firmware
//! - **Real-time Monitoring**: Monitor router and module events
//! - **Environment Variables**: Auto-configure from environment variables
//! - **Type-safe API**: Strongly-typed responses and error handling
//!
//! # Quick Start
//!
//! ```no_run
//! use bjig_controller::BjigController;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create controller (reads from environment variables)
//!     let bjig = BjigController::from_env()?;
//!
//!     // Get router version
//!     let version = bjig.router().get_version().await?;
//!     println!("Router version: {}", version.version);
//!
//!     // Get sensor data from module
//!     let data = bjig.module("0121", "2468800203400004")
//!         .instant_uplink()
//!         .await?;
//!     println!("Sensor data: {:?}", data);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables
//!
//! The library supports the following environment variables:
//!
//! - `BJIG_CLI_BIN_PATH` - Path to bjig binary (default: "./bin/bjig")
//! - `BJIG_CLI_PORT` - Serial port (e.g., "/dev/ttyACM0")
//! - `BJIG_CLI_BAUD` - Baud rate (default: 38400)
//! - `BJIG_CLI_MODULE_CONFIG` - Module config file path (default: "module-config.yml")
//!
//! # Examples
//!
//! See the `examples/` directory for more usage examples.

pub mod controller;
pub mod commands;
pub mod env;
pub mod executor;
pub mod types;

// Re-export main types
pub use controller::BjigController;
pub use types::*;

// Re-export environment constants for user reference
pub use env::{
    ENV_BJIG_CLI_BIN_PATH, ENV_BJIG_CLI_PORT, ENV_BJIG_CLI_BAUD,
    ENV_BJIG_CLI_MODULE_CONFIG, DEFAULT_BAUD, DEFAULT_MODULE_CONFIG,
    DEFAULT_BJIG_BINARY,
};
