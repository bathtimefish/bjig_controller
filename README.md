# bjig_controller

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](https://crates.io/crates/bjig_controller)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

[English](README.md) | [日本語](README_ja.md)

Rust library for controlling BraveJIG USB Router via `bjig` CLI command.

## Overview

`bjig_controller` provides a high-level, type-safe Rust API to control BraveJIG USB Router and sensor modules. It wraps the `bjig` CLI binary, making it easy to integrate BraveJIG functionality into Rust applications.

### What is BraveJIG?

BraveJIG is an IoT gateway system consisting of a USB router and wireless sensor modules. This library provides programmatic access to all router and module operations.

## Features

- **Router Control**: Start/stop router, manage firmware, configure settings
- **Module Management**: Control sensor modules, retrieve data, update firmware
- **Real-time Monitoring**: Monitor router and module events
- **Environment Variables**: Auto-configure from environment variables
- **Type-safe API**: Strongly-typed responses and comprehensive error handling
- **Async/Await**: Built on `tokio` for efficient async operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bjig_controller = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use bjig_controller::BjigController;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create controller (reads from environment variables)
    let bjig = BjigController::from_env()?;

    // Get router version
    let version = bjig.router().get_version().await?;
    println!("Router version: {}", version.version);

    // Get sensor data from module
    let data = bjig.module("0121", "2468800203400004")
        .instant_uplink()
        .await?;
    println!("Sensor data: {:?}", data);

    Ok(())
}
```

## Environment Variables

The library supports the following environment variables for configuration:

| Variable | Description | Default |
|----------|-------------|---------|
| `BJIG_CLI_BIN_PATH` | Path to bjig binary | `./bin/bjig` |
| `BJIG_CLI_PORT` | Serial port (e.g., `/dev/ttyACM0`, `COM3`) | *(required)* |
| `BJIG_CLI_BAUD` | Baud rate | `38400` |
| `BJIG_CLI_MODULE_CONFIG` | Module config file path | `module-config.yml` |

### Example

```bash
# Set environment variables
export BJIG_CLI_BIN_PATH=./bin/bjig
export BJIG_CLI_PORT=/dev/ttyACM0
export BJIG_CLI_BAUD=115200

# Run your application
cargo run
```

## Usage

### Creating a Controller

There are several ways to initialize `BjigController` depending on your needs:

#### 1. From Environment Variables (Recommended)

Automatically reads configuration from environment variables:

```rust
let bjig = BjigController::from_env()?;
```

This method reads:
- `BJIG_CLI_BIN_PATH` → bjig binary path (default: `./bin/bjig`)
- `BJIG_CLI_PORT` → serial port (required)
- `BJIG_CLI_BAUD` → baud rate (default: `38400`)

#### 2. Full Explicit Configuration

Specify all settings directly in code:

```rust
let bjig = BjigController::new("./bin/bjig")?
    .with_port("/dev/ttyACM0")
    .with_baud(115200);
```

#### 3. Minimal Configuration (Port Only)

Specify only the port, use defaults for everything else:

```rust
let bjig = BjigController::new("./bin/bjig")?
    .with_port("/dev/ttyACM0");
// Baud rate defaults to 38400
```

#### 4. Hybrid Approach

Mix environment variables with explicit overrides:

```rust
// Use env vars but override specific settings
let bjig = BjigController::from_env()?
    .with_baud(115200)
    .with_port("/dev/ttyACM1");
```

#### 5. Custom Binary Path

Use a custom bjig binary location:

```rust
let bjig = BjigController::new("/usr/local/bin/bjig")?
    .with_port("/dev/ttyACM0")
    .with_baud(115200);
```

#### 6. With Module Config

Specify a custom module configuration file:

```rust
let bjig = BjigController::from_env()?
    .with_module_config_path("/etc/bjig/custom-modules.yml");
```

### Configuration Priority

When the same setting is specified in multiple places, the priority is:

1. **Explicit method calls** (`.with_port()`, `.with_baud()`) - Highest priority
2. **Controller defaults** (set via builder methods)
3. **Environment variables** (when using `from_env()`)
4. **Built-in defaults** (38400 for baud rate, `./bin/bjig` for binary path)

Example:

```rust
// BJIG_CLI_PORT=/dev/ttyACM0 (environment)
// BJIG_CLI_BAUD=9600 (environment)

let bjig = BjigController::from_env()?
    .with_baud(115200);  // Override baud to 115200

// Result: port=/dev/ttyACM0 (from env), baud=115200 (explicit)

### Router Commands

```rust
// Get firmware version
let version = bjig.router().get_version().await?;

// Start/stop router
bjig.router().start().await?;
bjig.router().stop().await?;

// Get module IDs
let modules = bjig.router().get_module_id(None).await?; // All modules
let module = bjig.router().get_module_id(Some(0)).await?; // Specific index

// Configure scan mode
use bjig_controller::ScanModeType;
bjig.router().set_scan_mode(ScanModeType::LongRange).await?;

// Remove module ID
bjig.router().remove_module_id(Some(0)).await?; // Remove index 0
bjig.router().remove_module_id(None).await?; // Remove all

// Keep-alive signal
bjig.router().keep_alive().await?;

// Get supported sensors (no serial connection required)
let sensors = bjig.router().get_supported_sensor_id()?;

// Router firmware update
bjig.router().dfu("router_firmware.bin").await?;
```

### Module Commands

```rust
let module = bjig.module("0121", "2468800203400004");

// Get instant sensor data
let data = module.instant_uplink().await?;

// Get/set parameters
let params = module.get_parameter().await?;
module.set_parameter(&json!({"interval": 60})).await?;

// Restart module
module.restart().await?;

// Module firmware update
module.dfu("module_firmware.bin").await?;

// Module-specific control
module.control(&json!({"clear_counts": "all"})).await?;
```

### Monitor Command

```rust
// Monitor indefinitely (until Ctrl+C)
bjig.monitor().start().await?;

// Monitor with timeout (60 seconds)
bjig.monitor().start_with_ttl(60).await?;
```

### Port Override

All commands support optional port/baud overrides:

```rust
// Use default port/baud
let version = bjig.router().get_version().await?;

// Override for specific command
let version = bjig.router()
    .get_version_on(Some("/dev/ttyACM1"), Some(9600))
    .await?;
```

## Examples

See the `examples/` directory for complete examples:

- **`simple_router.rs`** - Basic router operations (version, scan mode, module list)
- **`module_control.rs`** - Module management (instant uplink, parameters)
- **`env_config.rs`** - Environment variable configuration
- **`restart_router.rs`** - Router restart sequence with timing control
- **`monitor.rs`** - Monitor uplink data continuously (until Ctrl+C)
- **`monitor_with_ttl.rs`** - Monitor uplink data with timeout

Run examples with:

```bash
# Make sure to set environment variables first
export BJIG_CLI_PORT=/dev/ttyACM0
export BJIG_CLI_BAUD=115200

# Run examples
cargo run --example simple_router
cargo run --example restart_router
cargo run --example monitor              # Press Ctrl+C to stop
cargo run --example monitor_with_ttl     # Stops after 30 seconds
```

## Error Handling

The library uses `Result<T, BjigError>` for all operations:

```rust
match bjig.router().get_version().await {
    Ok(version) => println!("Version: {}", version.version),
    Err(e) => eprintln!("Error: {}", e),
}
```

Common error types:

- `BinaryNotFound` - bjig binary not found at specified path
- `CommandFailed` - Command execution failed
- `PortNotConfigured` - Serial port not configured
- `JsonParseError` - Failed to parse command output
- `FileNotFound` - Firmware file not found

## Serial Port Exclusivity

The bjig command uses serial port communication, which is inherently exclusive. Only one process can connect to a serial port at a time. If you attempt to run multiple bjig_controller instances on the same port simultaneously, the second instance will fail with "connection busy" error.

This is a hardware limitation, not a library limitation.

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

## Requirements

- **Rust**: 1.70 or later
- **bjig CLI**: Binary included in `./bin/bjig`
- **Hardware**: BraveJIG USB Router and sensor modules
- **OS**: Linux, macOS, Windows (any platform supported by tokio)

## Architecture

```
┌─────────────────────────┐
│   Your Rust App         │
│                         │
├─────────────────────────┤
│   bjig_controller       │  ← This library (Type-safe API)
│   - Router Commands     │
│   - Module Commands     │
│   - Monitor Commands    │
├─────────────────────────┤
│   bjig CLI binary       │  ← Command-line tool
│   (process execution)   │
├─────────────────────────┤
│   BraveJIG Router       │  ← Hardware (USB connection)
│   + Sensor Modules      │
└─────────────────────────┘
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Author

**bathtimefish**

## Links

- **Repository**: https://github.com/bathtimefish/bjig_controller
- **Documentation**: https://docs.rs/bjig_controller
- **Crates.io**: https://crates.io/crates/bjig_controller
- **Issues**: https://github.com/bathtimefish/bjig_controller/issues

## Related Projects

- **bjig_cli_rust**: Official BraveJIG CLI tool (Rust implementation)
- **bjig_cli_python**: Official BraveJIG CLI tool (Python implementation)
