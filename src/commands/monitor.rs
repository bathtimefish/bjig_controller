//! Monitor command implementation

use crate::controller::BjigController;
use crate::executor::CommandExecutor;
use crate::types::Result;
use tokio::sync::mpsc;

/// Control messages for monitor process
#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlMessage {
    Stop,
    Pause,
    Resume,
}

/// Handle for controlling a running monitor process
///
/// This handle allows external control of a monitor process, including
/// graceful shutdown, pause, and resume. The monitor will automatically
/// stop when the handle is dropped or when explicitly stopped via `stop()`.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use bjig_controller::BjigController;
///
/// let bjig = BjigController::from_env()?;
///
/// // Start monitor with handle
/// let handle = bjig.monitor().start_with_handle().await?;
///
/// // Pause monitor (stops callback processing)
/// handle.pause().await?;
///
/// // Do some work...
///
/// // Resume monitor
/// handle.resume().await?;
///
/// // Stop monitor gracefully
/// handle.stop().await?;
/// # Ok(())
/// # }
/// ```
pub struct MonitorHandle {
    control_tx: mpsc::Sender<ControlMessage>,
    task_handle: tokio::task::JoinHandle<Result<()>>,
}

impl MonitorHandle {
    /// Pause the monitor
    ///
    /// This pauses callback processing. Data from the monitor process continues
    /// to be received but callbacks are not invoked. When resumed, processing
    /// continues with new data.
    ///
    /// # Errors
    ///
    /// Returns an error if the control channel is closed.
    pub async fn pause(&self) -> Result<()> {
        self.control_tx
            .send(ControlMessage::Pause)
            .await
            .map_err(|_| crate::types::BjigError::CommandFailed("Failed to send pause signal".to_string()))?;
        log::debug!("Pause signal sent to monitor");
        Ok(())
    }

    /// Resume the monitor
    ///
    /// This resumes callback processing after a pause. The monitor will
    /// continue processing new data received from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the control channel is closed.
    pub async fn resume(&self) -> Result<()> {
        self.control_tx
            .send(ControlMessage::Resume)
            .await
            .map_err(|_| crate::types::BjigError::CommandFailed("Failed to send resume signal".to_string()))?;
        log::debug!("Resume signal sent to monitor");
        Ok(())
    }

    /// Stop the monitor gracefully
    ///
    /// This sends a stop signal to the monitor process and waits for it
    /// to terminate. The method consumes the handle to ensure it can only
    /// be called once.
    ///
    /// # Errors
    ///
    /// Returns an error if the monitor task panicked or failed.
    pub async fn stop(mut self) -> Result<()> {
        // Send stop signal (ignore error if already stopped)
        let _ = self.control_tx.send(ControlMessage::Stop).await;

        // Wait for task to complete
        match (&mut self.task_handle).await {
            Ok(result) => result,
            Err(e) => {
                log::error!("Monitor task panicked: {}", e);
                Err(crate::types::BjigError::CommandFailed(format!("Monitor task panicked: {}", e)))
            }
        }
    }

    /// Check if monitor is still running
    ///
    /// Returns `true` if the monitor process is still active, `false` otherwise.
    pub fn is_running(&self) -> bool {
        !self.task_handle.is_finished()
    }
}

impl Drop for MonitorHandle {
    fn drop(&mut self) {
        // Send stop signal when handle is dropped (fire and forget)
        let _ = self.control_tx.try_send(ControlMessage::Stop);
    }
}

/// Monitor command interface
///
/// Provides real-time monitoring of router and module events.
/// The monitor command runs until interrupted (Ctrl+C) or until TTL expires.
pub struct MonitorCommand<'a> {
    controller: &'a BjigController,
}

impl<'a> MonitorCommand<'a> {
    /// Create new monitor command interface
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

    /// Start real-time monitoring (runs until Ctrl+C)
    ///
    /// This is a blocking operation that monitors router and module events
    /// in real-time. It will run indefinitely until interrupted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Monitor indefinitely (until Ctrl+C)
    /// bjig.monitor().start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<()> {
        self.start_on_impl(None, None, None).await
    }

    /// Start monitoring on specific port
    pub async fn start_on(&self, port: &str, baud: u32) -> Result<()> {
        self.start_on_impl(Some(port), Some(baud), None).await
    }

    /// Start monitoring with TTL (time-to-live in seconds)
    ///
    /// The monitoring will automatically stop after the specified duration.
    ///
    /// # Arguments
    /// * `ttl_secs` - Time to live in seconds
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Monitor for 60 seconds
    /// bjig.monitor().start_with_ttl(60).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_ttl(&self, ttl_secs: u64) -> Result<()> {
        self.start_on_impl(None, None, Some(ttl_secs)).await
    }

    /// Start monitoring on specific port with TTL
    pub async fn start_with_ttl_on(&self, port: &str, baud: u32, ttl_secs: u64) -> Result<()> {
        self.start_on_impl(Some(port), Some(baud), Some(ttl_secs))
            .await
    }

    /// Start monitoring with a callback for each JSON line
    ///
    /// The callback is called for each line received from the monitor.
    /// If the callback returns Ok(false), monitoring stops.
    ///
    /// # Arguments
    /// * `callback` - Function called for each line. Returns Ok(true) to continue, Ok(false) to stop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    /// let mut count = 0;
    ///
    /// bjig.monitor().start_with_callback(|line| {
    ///     println!("Received: {}", line);
    ///     count += 1;
    ///     Ok(count < 5) // Stop after 5 lines
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_callback<F>(&self, callback: F) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        self.start_with_callback_on_impl(None, None, None, callback)
            .await
    }

    /// Start monitoring on specific port with callback
    pub async fn start_with_callback_on<F>(
        &self,
        port: &str,
        baud: u32,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        self.start_with_callback_on_impl(Some(port), Some(baud), None, callback)
            .await
    }

    /// Start monitoring with TTL and callback
    pub async fn start_with_ttl_and_callback<F>(
        &self,
        ttl_secs: u64,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        self.start_with_callback_on_impl(None, None, Some(ttl_secs), callback)
            .await
    }

    /// Start monitoring with handle for external control
    ///
    /// Returns a `MonitorHandle` that can be used to stop the monitor
    /// from external code. The monitor runs in a background task.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    /// use tokio::time::{sleep, Duration};
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Start monitor with handle
    /// let handle = bjig.monitor().start_with_handle().await?;
    ///
    /// // Let it run for 5 seconds
    /// sleep(Duration::from_secs(5)).await;
    ///
    /// // Stop gracefully
    /// handle.stop().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_handle(&self) -> Result<MonitorHandle> {
        self.start_with_handle_impl(None, None, None).await
    }

    /// Start monitoring on specific port with handle
    pub async fn start_with_handle_on(&self, port: &str, baud: u32) -> Result<MonitorHandle> {
        self.start_with_handle_impl(Some(port), Some(baud), None)
            .await
    }

    /// Start monitoring with TTL and handle
    ///
    /// # Arguments
    /// * `ttl_secs` - Time to live in seconds
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    ///
    /// let bjig = BjigController::from_env()?;
    ///
    /// // Monitor for max 60 seconds, but can be stopped early
    /// let handle = bjig.monitor().start_with_ttl_and_handle(60).await?;
    ///
    /// // Stop before TTL expires
    /// handle.stop().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_ttl_and_handle(&self, ttl_secs: u64) -> Result<MonitorHandle> {
        self.start_with_handle_impl(None, None, Some(ttl_secs))
            .await
    }

    /// Start monitoring with callback and handle
    ///
    /// Combines callback functionality with external control via handle.
    /// The callback receives each line and can stop monitoring by returning
    /// `Ok(false)`. The handle can also be used to stop from external code.
    ///
    /// # Arguments
    /// * `callback` - Function called for each line. Returns Ok(true) to continue, Ok(false) to stop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use bjig_controller::BjigController;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let bjig = BjigController::from_env()?;
    /// let count = Arc::new(Mutex::new(0));
    /// let count_clone = count.clone();
    ///
    /// let handle = bjig.monitor().start_with_callback_and_handle(move |line| {
    ///     println!("Received: {}", line);
    ///     let mut c = count_clone.lock().unwrap();
    ///     *c += 1;
    ///     Ok(true) // Continue
    /// }).await?;
    ///
    /// // Stop from external code when needed
    /// handle.stop().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_with_callback_and_handle<F>(&self, callback: F) -> Result<MonitorHandle>
    where
        F: FnMut(&str) -> Result<bool> + Send + 'static,
    {
        self.start_with_callback_and_handle_impl(None, None, None, callback)
            .await
    }

    /// Start monitoring on specific port with callback and handle
    pub async fn start_with_callback_and_handle_on<F>(
        &self,
        port: &str,
        baud: u32,
        callback: F,
    ) -> Result<MonitorHandle>
    where
        F: FnMut(&str) -> Result<bool> + Send + 'static,
    {
        self.start_with_callback_and_handle_impl(Some(port), Some(baud), None, callback)
            .await
    }

    /// Start monitoring with TTL, callback, and handle
    pub async fn start_with_ttl_callback_and_handle<F>(
        &self,
        ttl_secs: u64,
        callback: F,
    ) -> Result<MonitorHandle>
    where
        F: FnMut(&str) -> Result<bool> + Send + 'static,
    {
        self.start_with_callback_and_handle_impl(None, None, Some(ttl_secs), callback)
            .await
    }

    async fn start_on_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        ttl_secs: Option<u64>,
    ) -> Result<()> {
        let executor = self.executor();

        let mut args = vec!["monitor"];

        let ttl_str;
        if let Some(ttl) = ttl_secs {
            ttl_str = ttl.to_string();
            args.push("--ttl");
            args.push(&ttl_str);
        }

        // Monitor command outputs to stdout continuously in real-time
        // Use execute_streaming to print each line as it arrives
        executor.execute_streaming(&args, port, baud).await?;

        Ok(())
    }

    async fn start_with_callback_on_impl<F>(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        ttl_secs: Option<u64>,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        let executor = self.executor();

        let mut args = vec!["monitor"];

        let ttl_str;
        if let Some(ttl) = ttl_secs {
            ttl_str = ttl.to_string();
            args.push("--ttl");
            args.push(&ttl_str);
        }

        // Monitor command outputs to stdout continuously in real-time
        // Use execute_streaming_with_callback to handle each line
        executor
            .execute_streaming_with_callback(&args, port, baud, callback)
            .await?;

        Ok(())
    }

    async fn start_with_handle_impl(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        ttl_secs: Option<u64>,
    ) -> Result<MonitorHandle> {
        // Clone necessary data to move into task
        let bjig_path = self.controller.bjig_path.clone();
        let default_port = self.controller.default_port.clone();
        let default_baud = self.controller.default_baud;
        let port_owned = port.map(|s| s.to_string());

        // Create channel for control signals
        let (control_tx, control_rx) = mpsc::channel(10);

        // Spawn monitor task
        let task_handle = tokio::spawn(async move {
            let executor = CommandExecutor::new(
                &bjig_path,
                default_port.as_deref(),
                default_baud,
            );

            let mut args_vec = vec!["monitor".to_string()];
            if let Some(ttl) = ttl_secs {
                args_vec.push("--ttl".to_string());
                args_vec.push(ttl.to_string());
            }
            let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

            executor
                .execute_streaming_with_control(&args, port_owned.as_deref(), baud, control_rx)
                .await
        });

        Ok(MonitorHandle {
            control_tx,
            task_handle,
        })
    }

    async fn start_with_callback_and_handle_impl<F>(
        &self,
        port: Option<&str>,
        baud: Option<u32>,
        ttl_secs: Option<u64>,
        callback: F,
    ) -> Result<MonitorHandle>
    where
        F: FnMut(&str) -> Result<bool> + Send + 'static,
    {
        // Clone necessary data to move into task
        let bjig_path = self.controller.bjig_path.clone();
        let default_port = self.controller.default_port.clone();
        let default_baud = self.controller.default_baud;
        let port_owned = port.map(|s| s.to_string());

        // Create channel for control signals
        let (control_tx, control_rx) = mpsc::channel(10);

        // Spawn monitor task
        let task_handle = tokio::spawn(async move {
            let executor = CommandExecutor::new(
                &bjig_path,
                default_port.as_deref(),
                default_baud,
            );

            let mut args_vec = vec!["monitor".to_string()];
            if let Some(ttl) = ttl_secs {
                args_vec.push("--ttl".to_string());
                args_vec.push(ttl.to_string());
            }
            let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

            executor
                .execute_streaming_with_callback_and_control(&args, port_owned.as_deref(), baud, callback, control_rx)
                .await
        });

        Ok(MonitorHandle {
            control_tx,
            task_handle,
        })
    }
}
