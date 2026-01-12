//! Monitor command implementation

use crate::controller::BjigController;
use crate::executor::CommandExecutor;
use crate::types::Result;

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
}
