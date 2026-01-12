//! Core BjigController implementation

use std::path::{Path, PathBuf};

use crate::commands::{MonitorCommand, ModuleCommands, RouterCommands};
use crate::env;
use crate::types::{BjigError, Result};

/// Main controller for bjig CLI operations
///
/// This controller provides a high-level interface to all bjig commands.
/// It manages configuration (port, baud rate, bjig binary path) and provides
/// access to router, module, and monitor command interfaces.
///
/// # Examples
///
/// ```no_run
/// use bjig_controller::BjigController;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create controller with environment variables
///     let bjig = BjigController::from_env()?;
///
///     // Get router version
///     let version = bjig.router().get_version().await?;
///     println!("Version: {}", version.version);
///
///     Ok(())
/// }
/// ```
pub struct BjigController {
    pub(crate) bjig_path: PathBuf,
    pub(crate) default_port: Option<String>,
    pub(crate) default_baud: Option<u32>,
    pub(crate) module_config_path: Option<PathBuf>,
}

impl BjigController {
    /// Create new controller with explicit bjig binary path
    ///
    /// # Arguments
    /// * `bjig_path` - Path to bjig binary
    ///
    /// # Errors
    /// Returns `BjigError::BinaryNotFound` if the binary doesn't exist
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::new("./bin/bjig")?;
    /// # Ok::<(), bjig_controller::BjigError>(())
    /// ```
    pub fn new<P: AsRef<Path>>(bjig_path: P) -> Result<Self> {
        let path = bjig_path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(BjigError::BinaryNotFound(path));
        }

        Ok(Self {
            bjig_path: path,
            default_port: None,
            default_baud: None,
            module_config_path: None,
        })
    }

    /// Create controller with auto-detection from environment variables
    ///
    /// This method reads configuration from environment variables:
    /// - `BJIG_CLI_BIN_PATH` - bjig binary path (default: "./bin/bjig")
    /// - `BJIG_CLI_PORT` - serial port
    /// - `BJIG_CLI_BAUD` - baud rate
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// // Reads from BJIG_CLI_* environment variables
    /// let bjig = BjigController::from_env()?;
    /// # Ok::<(), bjig_controller::BjigError>(())
    /// ```
    pub fn from_env() -> Result<Self> {
        let bjig_path = env::get_bjig_binary_path();
        let mut controller = Self::new(bjig_path)?;

        // Auto-load from environment
        if let Some(port) = env::get_port_from_env() {
            controller.default_port = Some(port);
        }
        if let Some(baud) = env::get_baud_from_env() {
            controller.default_baud = Some(baud);
        }

        Ok(controller)
    }

    /// Set default serial port
    ///
    /// This port will be used for all commands unless overridden.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::new("./bin/bjig")?
    ///     .with_port("/dev/ttyACM0");
    /// # Ok::<(), bjig_controller::BjigError>(())
    /// ```
    pub fn with_port(mut self, port: impl Into<String>) -> Self {
        self.default_port = Some(port.into());
        self
    }

    /// Set default baud rate
    ///
    /// This baud rate will be used for all commands unless overridden.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::new("./bin/bjig")?
    ///     .with_baud(115200);
    /// # Ok::<(), bjig_controller::BjigError>(())
    /// ```
    pub fn with_baud(mut self, baud: u32) -> Self {
        self.default_baud = Some(baud);
        self
    }

    /// Set module config file path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::new("./bin/bjig")?
    ///     .with_module_config_path("/etc/bjig/modules.yml");
    /// # Ok::<(), bjig_controller::BjigError>(())
    /// ```
    pub fn with_module_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.module_config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Get router commands interface
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let bjig = BjigController::from_env()?;
    /// let version = bjig.router().get_version().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn router(&self) -> RouterCommands {
        RouterCommands::new(self)
    }

    /// Get module commands interface for specific module
    ///
    /// # Arguments
    /// * `sensor_id` - Sensor ID (e.g., "0121" for illuminance sensor)
    /// * `module_id` - Module ID (16-digit hex string)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let bjig = BjigController::from_env()?;
    /// let data = bjig.module("0121", "2468800203400004")
    ///     .instant_uplink()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn module(&self, sensor_id: &str, module_id: &str) -> ModuleCommands {
        ModuleCommands::new(self, sensor_id, module_id)
    }

    /// Get monitor command interface
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bjig_controller::BjigController;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let bjig = BjigController::from_env()?;
    /// bjig.monitor().start_with_ttl(60).await?;  // Monitor for 60 seconds
    /// # Ok(())
    /// # }
    /// ```
    pub fn monitor(&self) -> MonitorCommand {
        MonitorCommand::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_builder() {
        let bjig = BjigController::new("./bin/bjig")
            .unwrap()
            .with_port("/dev/ttyACM0")
            .with_baud(115200);

        assert_eq!(bjig.default_port, Some("/dev/ttyACM0".to_string()));
        assert_eq!(bjig.default_baud, Some(115200));
    }
}
