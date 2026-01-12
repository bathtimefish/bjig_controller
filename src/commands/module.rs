//! Module command implementations

use std::path::Path;

use crate::controller::BjigController;
use crate::executor::CommandExecutor;
use crate::types::*;

/// Module commands interface
///
/// Provides access to all module-related operations including:
/// - Instant uplink (sensor data retrieval)
/// - Parameter management
/// - Module restart
/// - Firmware updates (DFU)
/// - Module-specific control commands
pub struct ModuleCommands<'a> {
    controller: &'a BjigController,
    sensor_id: String,
    module_id: String,
}

impl<'a> ModuleCommands<'a> {
    /// Create new module commands interface
    pub(crate) fn new(controller: &'a BjigController, sensor_id: &str, module_id: &str) -> Self {
        Self {
            controller,
            sensor_id: sensor_id.to_string(),
            module_id: module_id.to_string(),
        }
    }

    /// Get command executor
    fn executor(&self) -> CommandExecutor {
        CommandExecutor::new(
            &self.controller.bjig_path,
            self.controller.default_port.as_deref(),
            self.controller.default_baud,
        )
    }

    /// Request instant uplink (immediate sensor data retrieval)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let data = bjig.module("0121", "2468800203400004")
    ///     .instant_uplink()
    ///     .await?;
    /// println!("Sensor data: {:?}", data);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn instant_uplink(&self) -> Result<serde_json::Value> {
        self.instant_uplink_with_timeout(30).await
    }

    /// Request instant uplink on specific port
    pub async fn instant_uplink_on(
        &self,
        port: &str,
        baud: u32,
    ) -> Result<serde_json::Value> {
        self.instant_uplink_with_timeout_on(port, baud, 30).await
    }

    /// Request instant uplink with custom timeout
    ///
    /// # Arguments
    /// * `timeout_secs` - Response timeout in seconds (default: 30)
    pub async fn instant_uplink_with_timeout(
        &self,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        self.instant_uplink_with_timeout_on_impl(None, None, timeout_secs)
            .await
    }

    /// Request instant uplink on specific port with custom timeout
    pub async fn instant_uplink_with_timeout_on(
        &self,
        port: &str,
        baud: u32,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        self.instant_uplink_with_timeout_on_impl(Some(port), Some(baud), timeout_secs)
            .await
    }

    async fn instant_uplink_with_timeout_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        let executor = self.executor();
        let timeout_str = timeout_secs.to_string();

        let args = vec![
            "module",
            "instant-uplink",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--response-timeout",
            &timeout_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(json)
    }

    /// Get module parameters
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let params = bjig.module("0121", "2468800203400004")
    ///     .get_parameter()
    ///     .await?;
    /// println!("Parameters: {:?}", params);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_parameter(&self) -> Result<serde_json::Value> {
        self.get_parameter_with_timeout(30).await
    }

    /// Get module parameters on specific port
    pub async fn get_parameter_on(
        &self,
        port: &str,
        baud: u32,
    ) -> Result<serde_json::Value> {
        self.get_parameter_with_timeout_on(port, baud, 30).await
    }

    /// Get module parameters with custom timeout
    pub async fn get_parameter_with_timeout(
        &self,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        self.get_parameter_with_timeout_on_impl(None, None, timeout_secs)
            .await
    }

    /// Get module parameters on specific port with custom timeout
    pub async fn get_parameter_with_timeout_on(
        &self,
        port: &str,
        baud: u32,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        self.get_parameter_with_timeout_on_impl(Some(port), Some(baud), timeout_secs)
            .await
    }

    async fn get_parameter_with_timeout_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        timeout_secs: u64,
    ) -> Result<serde_json::Value> {
        let executor = self.executor();
        let timeout_str = timeout_secs.to_string();

        let args = vec![
            "module",
            "get-parameter",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--response-timeout",
            &timeout_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(json)
    }

    /// Set module parameters
    ///
    /// # Arguments
    /// * `data` - Parameter data as JSON
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    /// use serde_json::json;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let params = json!({
    ///     "interval": 60,
    ///     "threshold": 100
    /// });
    ///
    /// let result = bjig.module("0121", "2468800203400004")
    ///     .set_parameter(&params)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_parameter(&self, data: &serde_json::Value) -> Result<SetParameterResult> {
        self.set_parameter_with_timeout(data, 30).await
    }

    /// Set module parameters on specific port
    pub async fn set_parameter_on(
        &self,
        port: &str,
        baud: u32,
        data: &serde_json::Value,
    ) -> Result<SetParameterResult> {
        self.set_parameter_with_timeout_on(port, baud, data, 30)
            .await
    }

    /// Set module parameters with custom timeout
    pub async fn set_parameter_with_timeout(
        &self,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<SetParameterResult> {
        self.set_parameter_with_timeout_on_impl(None, None, data, timeout_secs)
            .await
    }

    /// Set module parameters on specific port with custom timeout
    pub async fn set_parameter_with_timeout_on(
        &self,
        port: &str,
        baud: u32,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<SetParameterResult> {
        self.set_parameter_with_timeout_on_impl(Some(port), Some(baud), data, timeout_secs)
            .await
    }

    async fn set_parameter_with_timeout_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<SetParameterResult> {
        let executor = self.executor();
        let data_str = serde_json::to_string(data)?;
        let timeout_str = timeout_secs.to_string();

        let args = vec![
            "module",
            "set-parameter",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--data",
            &data_str,
            "--response-timeout",
            &timeout_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(serde_json::from_value(json)?)
    }

    /// Restart module
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.module("0121", "2468800203400004")
    ///     .restart()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restart(&self) -> Result<RestartResult> {
        self.restart_with_timeout(30).await
    }

    /// Restart module on specific port
    pub async fn restart_on(&self, port: &str, baud: u32) -> Result<RestartResult> {
        self.restart_with_timeout_on(port, baud, 30).await
    }

    /// Restart module with custom timeout
    pub async fn restart_with_timeout(&self, timeout_secs: u64) -> Result<RestartResult> {
        self.restart_with_timeout_on_impl(None, None, timeout_secs)
            .await
    }

    /// Restart module on specific port with custom timeout
    pub async fn restart_with_timeout_on(
        &self,
        port: &str,
        baud: u32,
        timeout_secs: u64,
    ) -> Result<RestartResult> {
        self.restart_with_timeout_on_impl(Some(port), Some(baud), timeout_secs)
            .await
    }

    async fn restart_with_timeout_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        timeout_secs: u64,
    ) -> Result<RestartResult> {
        let executor = self.executor();
        let timeout_str = timeout_secs.to_string();

        let args = vec![
            "module",
            "restart",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--response-timeout",
            &timeout_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(serde_json::from_value(json)?)
    }

    /// Module DFU (firmware update)
    ///
    /// # Arguments
    /// * `firmware_path` - Path to module firmware file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let result = bjig.module("0121", "2468800203400004")
    ///     .dfu("module_firmware.bin")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dfu<P: AsRef<Path>>(&self, firmware_path: P) -> Result<DfuResult> {
        self.dfu_on_impl(None, None, firmware_path).await
    }

    /// Module DFU on specific port
    pub async fn dfu_on<P: AsRef<Path>>(
        &self,
        port: &str,
        baud: u32,
        firmware_path: P,
    ) -> Result<DfuResult> {
        self.dfu_on_impl(Some(port), Some(baud), firmware_path)
            .await
    }

    async fn dfu_on_impl<P: AsRef<Path>>(
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

        let args = vec![
            "module",
            "dfu",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--file",
            &path_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(serde_json::from_value(json)?)
    }

    /// Send module-specific control command
    ///
    /// # Arguments
    /// * `data` - Control data as JSON (module-specific schema)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    /// use serde_json::json;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // DryContact module: clear counts
    /// let control = json!({"clear_counts": "all"});
    /// let result = bjig.module("0126", "2468800203400004")
    ///     .control(&control)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn control(&self, data: &serde_json::Value) -> Result<ControlResult> {
        self.control_with_timeout(data, 30).await
    }

    /// Send control command on specific port
    pub async fn control_on(
        &self,
        port: &str,
        baud: u32,
        data: &serde_json::Value,
    ) -> Result<ControlResult> {
        self.control_with_timeout_on(port, baud, data, 30).await
    }

    /// Send control command with custom timeout
    pub async fn control_with_timeout(
        &self,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<ControlResult> {
        self.control_with_timeout_on_impl(None, None, data, timeout_secs)
            .await
    }

    /// Send control command on specific port with custom timeout
    pub async fn control_with_timeout_on(
        &self,
        port: &str,
        baud: u32,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<ControlResult> {
        self.control_with_timeout_on_impl(Some(port), Some(baud), data, timeout_secs)
            .await
    }

    async fn control_with_timeout_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        data: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<ControlResult> {
        let executor = self.executor();
        let data_str = serde_json::to_string(data)?;
        let timeout_str = timeout_secs.to_string();

        let args = vec![
            "module",
            "control",
            "--sensor-id",
            &self.sensor_id,
            "--module-id",
            &self.module_id,
            "--data",
            &data_str,
            "--response-timeout",
            &timeout_str,
        ];

        let json = executor.execute_json(&args, port, baud).await?;
        Ok(serde_json::from_value(json)?)
    }
}
