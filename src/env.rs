//! Environment variable handling for bjig_controller

use std::env;
use std::path::PathBuf;

use crate::types::{BjigError, Result};

/// Environment variable for bjig binary path
pub const ENV_BJIG_CLI_BIN_PATH: &str = "BJIG_CLI_BIN_PATH";

/// Environment variable for serial port
pub const ENV_BJIG_CLI_PORT: &str = "BJIG_CLI_PORT";

/// Environment variable for baud rate
pub const ENV_BJIG_CLI_BAUD: &str = "BJIG_CLI_BAUD";

/// Environment variable for module config file path
pub const ENV_BJIG_CLI_MODULE_CONFIG: &str = "BJIG_CLI_MODULE_CONFIG";

/// Default baud rate (matches bjig_cli_rust default)
pub const DEFAULT_BAUD: u32 = 38400;

/// Default module config file name
pub const DEFAULT_MODULE_CONFIG: &str = "module-config.yml";

/// Default bjig binary path (relative to crate root)
pub const DEFAULT_BJIG_BINARY: &str = "./bin/bjig";

/// Get bjig binary path from environment or default
///
/// Priority:
/// 1. BJIG_CLI_BIN_PATH environment variable
/// 2. DEFAULT_BJIG_BINARY ("./bin/bjig")
pub fn get_bjig_binary_path() -> PathBuf {
    env::var(ENV_BJIG_CLI_BIN_PATH)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BJIG_BINARY))
}

/// Get port from environment variable
pub fn get_port_from_env() -> Option<String> {
    env::var(ENV_BJIG_CLI_PORT).ok()
}

/// Get baud rate from environment variable
pub fn get_baud_from_env() -> Option<u32> {
    env::var(ENV_BJIG_CLI_BAUD)
        .ok()
        .and_then(|s| s.parse().ok())
}

/// Get module config path from environment or default
pub fn get_module_config_from_env() -> String {
    env::var(ENV_BJIG_CLI_MODULE_CONFIG).unwrap_or_else(|_| DEFAULT_MODULE_CONFIG.to_string())
}

/// Resolve port with priority: explicit > default > env
///
/// # Arguments
/// * `explicit` - Explicitly provided port (highest priority)
/// * `default` - Default port from controller (medium priority)
///
/// # Returns
/// Port string if found, otherwise PortNotConfigured error
pub fn resolve_port(explicit: Option<&str>, default: Option<&str>) -> Result<String> {
    explicit
        .map(String::from)
        .or_else(|| default.map(String::from))
        .or_else(get_port_from_env)
        .ok_or(BjigError::PortNotConfigured)
}

/// Resolve baud with priority: explicit > default > env > DEFAULT_BAUD
///
/// # Arguments
/// * `explicit` - Explicitly provided baud rate (highest priority)
/// * `default` - Default baud rate from controller (medium priority)
///
/// # Returns
/// Baud rate (always returns a value, using DEFAULT_BAUD as fallback)
pub fn resolve_baud(explicit: Option<u32>, default: Option<u32>) -> u32 {
    explicit
        .or(default)
        .or_else(get_baud_from_env)
        .unwrap_or(DEFAULT_BAUD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_port_explicit() {
        let result = resolve_port(Some("/dev/ttyACM0"), Some("/dev/ttyACM1")).unwrap();
        assert_eq!(result, "/dev/ttyACM0");
    }

    #[test]
    fn test_resolve_port_default() {
        let result = resolve_port(None, Some("/dev/ttyACM1")).unwrap();
        assert_eq!(result, "/dev/ttyACM1");
    }

    #[test]
    fn test_resolve_baud_explicit() {
        let result = resolve_baud(Some(115200), Some(9600));
        assert_eq!(result, 115200);
    }

    #[test]
    fn test_resolve_baud_default() {
        let result = resolve_baud(None, Some(9600));
        assert_eq!(result, 9600);
    }

    #[test]
    fn test_resolve_baud_fallback() {
        let result = resolve_baud(None, None);
        assert_eq!(result, DEFAULT_BAUD);
    }
}
