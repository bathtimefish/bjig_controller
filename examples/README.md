# Examples

This directory contains example programs demonstrating how to use the `bjig_controller` library.

## Prerequisites

Before running the examples, you need:

1. **BraveJIG USB Router** connected to your computer
2. **bjig CLI** installed and accessible in your PATH or specified via environment variable
3. **Environment variables** configured (see below)

## Environment Variables

Set the following environment variables before running the examples:

```bash
# Required: Serial port where BraveJIG router is connected
export BJIG_CLI_PORT=/dev/ttyACM0  # Linux/macOS
# or
# set BJIG_CLI_PORT=COM3            # Windows

# Required: Baud rate (usually 115200)
export BJIG_CLI_BAUD=115200

# Optional: Path to bjig CLI binary (if not in PATH)
export BJIG_CLI_BIN_PATH=/path/to/bjig

# Optional: Module configuration file
export BJIG_CLI_MODULE_CONFIG=./module-config.yml
```

## Running Examples

Each example can be run using `cargo run --example <example_name>`:

```bash
cargo run --example simple_router
```

Enable verbose logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run --example simple_router
```

## Available Examples

### 1. `simple_router.rs`

Basic router operations including getting firmware version, scan mode, and module IDs.

**What it demonstrates:**
- Creating a controller from environment variables
- Getting router version
- Getting scan mode
- Listing connected modules

**Usage:**
```bash
cargo run --example simple_router
```

### 2. `env_config.rs`

Demonstrates how to configure the controller using environment variables.

**What it demonstrates:**
- Reading configuration from environment variables
- Displaying current configuration
- Testing the configuration with a simple command

**Usage:**
```bash
cargo run --example env_config
```

### 3. `restart_router.rs`

Complete router restart sequence with proper timing and verification.

**What it demonstrates:**
- Stopping the router
- Waiting for proper shutdown
- Starting the router
- Verifying router is operational

**Usage:**
```bash
cargo run --example restart_router
```

**Note:** This example takes about 11 seconds to complete due to required wait times.

### 4. `module_control.rs`

Module-specific operations including instant uplink and parameter management.

**What it demonstrates:**
- Getting instant uplink data from a sensor
- Reading module parameters
- Working with sensor and module IDs

**Usage:**
```bash
# Edit the example to use your sensor_id and module_id
cargo run --example module_control
```

**Note:** You need to modify the `sensor_id` and `module_id` variables in the code to match your actual devices.

### 5. `monitor.rs`

Real-time monitoring of uplink data with callback processing.

**What it demonstrates:**
- Starting the monitor process
- Processing JSON uplink data line-by-line
- Using callbacks to handle data
- Collecting and displaying received data
- Stopping monitor after a condition is met (5 items collected)

**Usage:**
```bash
cargo run --example monitor
```

**Expected behavior:** The example will collect 5 JSON uplink messages and then automatically stop.

### 6. `monitor_with_ttl.rs`

Real-time monitoring with a timeout (TTL - Time To Live).

**What it demonstrates:**
- Starting the monitor process with a timeout
- Processing JSON uplink data with a time limit
- Using callbacks with TTL
- Stopping monitor after reaching either:
  - A collection limit (5 items), OR
  - A timeout (120 seconds)

**Usage:**
```bash
cargo run --example monitor_with_ttl
```

**Expected behavior:** The example will collect up to 5 JSON uplink messages or stop after 120 seconds, whichever comes first.

### 7. `monitor_with_handle.rs`

Real-time monitoring with external control via handle, demonstrating pause/resume functionality.

**What it demonstrates:**
- Starting monitor with a handle for external control
- Pausing monitor (callback processing stops, data buffered by router)
- Resuming monitor (buffered data is received)
- Stopping monitor gracefully
- Handle-based control allows flexible monitoring workflows

**Usage:**
```bash
cargo run --example monitor_with_handle
```

**Expected behavior:** Monitor runs for 5 seconds, pauses for 3 seconds, resumes for 5 seconds, then stops. During pause, no output is shown but the router continues buffering data.

### 8. `monitor_with_callback_and_handle.rs`

Combines callback processing with pause/resume control via handle.

**What it demonstrates:**
- Starting monitor with both callback and handle
- Processing each JSON line with a callback
- Counting messages received
- Pausing callback processing while data is buffered
- Resuming callback processing
- External control of callback-based monitoring

**Usage:**
```bash
cargo run --example monitor_with_callback_and_handle
```

**Expected behavior:** Receives 3 messages, pauses for 3 seconds (callback not invoked), resumes and receives 2 more messages, then stops. Total of 5 messages processed.

## Common Issues

### "No such file or directory" or "bjig not found"

Make sure the `bjig` CLI is installed and either in your PATH or specify its location:

```bash
export BJIG_CLI_BIN_PATH=/path/to/bjig
```

### "Permission denied" on serial port

On Linux/macOS, you may need to add your user to the dialout/uucp group:

```bash
# Linux
sudo usermod -a -G dialout $USER

# macOS
sudo dseditgroup -o edit -a $USER -t user uucp
```

Then log out and log back in for the changes to take effect.

### "No such device" error

Make sure your BraveJIG router is connected and check the correct port:

```bash
# Linux/macOS - list available ports
ls /dev/tty*

# macOS specifically
ls /dev/cu.*
```

## Learning Path

Recommended order for understanding the library:

1. Start with `env_config.rs` to understand configuration
2. Try `simple_router.rs` for basic operations
3. Experiment with `module_control.rs` if you have sensors connected
4. Use `restart_router.rs` to learn restart sequences
5. Explore `monitor.rs` for real-time data collection
6. Learn `monitor_with_ttl.rs` for timeout-based monitoring
7. Advanced: `monitor_with_handle.rs` for external control with pause/resume
8. Advanced: `monitor_with_callback_and_handle.rs` for callback-based monitoring with control

## Additional Resources

- [Main README](../README.md) - Library documentation
- [API Documentation](https://docs.rs/bjig_controller) - Full API reference
- [bjig CLI Documentation](https://github.com/MONO-ON/bravejig) - BraveJIG CLI tool
