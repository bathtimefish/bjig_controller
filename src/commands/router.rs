//! Router command implementations

use std::path::Path;

use crate::controller::BjigController;
use crate::executor::CommandExecutor;
use crate::types::*;

/// Router commands interface
///
/// Provides access to all router-related operations including:
/// - Router control (start/stop)
/// - Version information
/// - Module ID management
/// - Scan mode configuration
/// - Firmware updates (DFU)
pub struct RouterCommands<'a> {
    controller: &'a BjigController,
}

impl<'a> RouterCommands<'a> {
    /// Create new router commands interface
    pub(crate) fn new(controller: &'a BjigController) -> Self {
        Self { controller }
    }

    /// Get command executor
    fn executor(&self) -> CommandExecutor {
        CommandExecutor::new(
            &self.controller.bjig_path,
            self.controller.default_port.as_deref(),
            self.controller.default_baud,
        )
    }

    /// Start router
    ///
    /// Uses default port and baud rate configured in controller.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.router().start().await?;
    /// println!("Router started: {}", result.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<StartResult> {
        self.start_on(None, None).await
    }

    /// Start router on specific port
    ///
    /// # Arguments
    /// * `port` - Serial port (e.g., "/dev/ttyACM0")
    /// * `baud` - Baud rate (e.g., 115200)
    pub async fn start_on(&self, port: Option<&str>, baud: Option<u32>) -> Result<StartResult> {
        let executor = self.executor();
        let json = executor
            .execute_json(&["router", "start"], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Stop router
    ///
    /// Uses default port and baud rate configured in controller.
    pub async fn stop(&self) -> Result<StopResult> {
        self.stop_on(None, None).await
    }

    /// Stop router on specific port
    pub async fn stop_on(&self, port: Option<&str>, baud: Option<u32>) -> Result<StopResult> {
        let executor = self.executor();
        let json = executor
            .execute_json(&["router", "stop"], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Get router firmware version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let version = bjig.router().get_version().await?;
    /// println!("Router version: {}.{}.{}", version.major, version.minor, version.build);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_version(&self) -> Result<Version> {
        self.get_version_on(None, None).await
    }

    /// Get router firmware version on specific port
    pub async fn get_version_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
    ) -> Result<Version> {
        let executor = self.executor();
        let json = executor
            .execute_json(&["router", "get-version"], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Get module IDs
    ///
    /// # Arguments
    /// * `index` - Optional module index (0-99). If None, returns all module IDs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Get all module IDs
    /// let all_modules = bjig.router().get_module_id(None).await?;
    /// println!("Found {} modules", all_modules.modules.len());
    ///
    /// // Get specific module at index 0
    /// let module = bjig.router().get_module_id(Some(0)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_module_id(&self, index: Option<u8>) -> Result<ModuleIdList> {
        self.get_module_id_on(None, None, index).await
    }

    /// Get module IDs on specific port
    pub async fn get_module_id_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        index: Option<u8>,
    ) -> Result<ModuleIdList> {
        let executor = self.executor();

        let idx_str;
        let args = if let Some(idx) = index {
            idx_str = idx.to_string();
            vec!["router", "get-module-id", idx_str.as_str()]
        } else {
            vec!["router", "get-module-id"]
        };

        let json = executor.execute_json(&args, port, baud).await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Get scan mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let mode = bjig.router().get_scan_mode().await?;
    /// println!("Scan mode: {} ({})", mode.mode, mode.mode_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_scan_mode(&self) -> Result<ScanMode> {
        self.get_scan_mode_on(None, None).await
    }

    /// Get scan mode on specific port
    pub async fn get_scan_mode_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
    ) -> Result<ScanMode> {
        let executor = self.executor();
        let json = executor
            .execute_json(&["router", "get-scan-mode"], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Set scan mode
    ///
    /// # Arguments
    /// * `mode` - Scan mode (0 = LongRange, 1 = Legacy)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::{BjigController, ScanModeType};
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.router().set_scan_mode(ScanModeType::LongRange).await?;
    /// println!("Scan mode set: {}", result.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_scan_mode(&self, mode: ScanModeType) -> Result<SetScanModeResult> {
        self.set_scan_mode_on(None, None, mode).await
    }

    /// Set scan mode on specific port
    pub async fn set_scan_mode_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        mode: ScanModeType,
    ) -> Result<SetScanModeResult> {
        let executor = self.executor();
        let mode_str = mode.to_u8().to_string();
        let json = executor
            .execute_json(&["router", "set-scan-mode", &mode_str], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Remove module ID
    ///
    /// # Arguments
    /// * `index` - Optional module index (0-99). If None, removes all module IDs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Remove module at index 0
    /// let result = bjig.router().remove_module_id(Some(0)).await?;
    ///
    /// // Remove all modules
    /// let result = bjig.router().remove_module_id(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_module_id(&self, index: Option<u8>) -> Result<RemoveResult> {
        self.remove_module_id_on(None, None, index).await
    }

    /// Remove module ID on specific port
    pub async fn remove_module_id_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        index: Option<u8>,
    ) -> Result<RemoveResult> {
        let executor = self.executor();

        let idx_str;
        let args = if let Some(idx) = index {
            idx_str = idx.to_string();
            vec!["router", "remove-module-id", idx_str.as_str()]
        } else {
            vec!["router", "remove-module-id"]
        };

        let json = executor.execute_json(&args, port, baud).await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Send keep-alive signal (time synchronization)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.router().keep_alive().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn keep_alive(&self) -> Result<KeepAliveResult> {
        self.keep_alive_on(None, None).await
    }

    /// Send keep-alive signal on specific port
    pub async fn keep_alive_on(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
    ) -> Result<KeepAliveResult> {
        let executor = self.executor();
        let json = executor
            .execute_json(&["router", "keep-alive"], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }

    /// Get supported sensor IDs and capabilities (static, no serial connection required)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let sensors = bjig.router().get_supported_sensor_id()?;
    /// for sensor in sensors {
    ///     println!("{}: {}", sensor.sensor_id, sensor.sensor_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_supported_sensor_id(&self) -> Result<Vec<SensorInfo>> {
        let executor = self.executor();
        let json = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(executor.execute_static(&["router", "get-supported-sensor-id"]))
        })?;

        Ok(serde_json::from_value(json)?)
    }

    /// Get module configuration from YAML file (static, no serial connection required)
    ///
    /// # Arguments
    /// * `file_path` - Path to module config YAML file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let config = bjig.router().get_module_config("module-config.yml")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_module_config<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<ModuleConfig>> {
        let executor = self.executor();
        let path_str = file_path.as_ref().to_string_lossy();

        let json = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                executor.execute_static(&["router", "get-module-config", "--file", &path_str]),
            )
        })?;

        Ok(serde_json::from_value(json)?)
    }

    /// Router DFU (firmware update)
    ///
    /// # Arguments
    /// * `firmware_path` - Path to firmware file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.router().dfu("router_firmware.bin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dfu<P: AsRef<Path>>(&self, firmware_path: P) -> Result<DfuResult> {
        self.dfu_on(None, None, firmware_path).await
    }

    /// Router DFU on specific port
    pub async fn dfu_on<P: AsRef<Path>>(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        firmware_path: P,
    ) -> Result<DfuResult> {
        let path = firmware_path.as_ref();

        if !path.exists() {
            return Err(BjigError::FileNotFound(path.to_path_buf()));
        }

        let executor = self.executor();
        let path_str = path.to_string_lossy();

        let json = executor
            .execute_json(&["router", "dfu", "--file", &path_str], port, baud)
            .await?;

        Ok(serde_json::from_value(json)?)
    }
}
