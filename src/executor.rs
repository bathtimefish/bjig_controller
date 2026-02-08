//! Command executor for running bjig binary

use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::commands::monitor::ControlMessage;
use crate::env::{resolve_baud, resolve_port};
use crate::types::{BjigError, Result};

/// Command executor that handles bjig binary execution
pub(crate) struct CommandExecutor<'a> {
    pub bjig_path: &'a Path,
    pub default_port: Option<&'a str>,
    pub default_baud: Option<u32>,
}

impl<'a> CommandExecutor<'a> {
    /// Create new executor
    pub fn new(
        bjig_path: &'a Path,
        default_port: Option<&'a str>,
        default_baud: Option<u32>,
    ) -> Self {
        Self {
            bjig_path,
            default_port,
            default_baud,
        }
    }

    /// Execute bjig command and parse JSON output
    ///
    /// # Arguments
    /// * `args` - Command arguments (without --port and --baud, added automatically)
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    pub async fn execute_json(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
    ) -> Result<serde_json::Value> {
        let full_args = self.build_args(args, port_override, baud_override)?;
        let output = self.run_command(&full_args).await?;

        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
            log::error!("Failed to parse JSON output: {}", output);
            e
        })?;

        Ok(json)
    }

    /// Execute bjig command without port/baud (for static commands)
    ///
    /// # Arguments
    /// * `args` - Command arguments
    pub async fn execute_static(&self, args: &[&str]) -> Result<serde_json::Value> {
        let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let output = self.run_command(&args_vec).await?;

        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&output)?;

        Ok(json)
    }

    /// Execute bjig command and stream stdout line by line
    ///
    /// This is used for commands that produce continuous output (like monitor).
    /// The output is printed to stdout in real-time.
    pub async fn execute_streaming(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
    ) -> Result<()> {
        self.execute_streaming_with_callback(args, port_override, baud_override, |line| {
            println!("{}", line);
            Ok(true) // Continue streaming
        })
        .await
    }

    /// Execute bjig command and stream stdout line by line with callback
    ///
    /// This is used for commands that produce continuous output (like monitor).
    /// The callback is called for each line. If the callback returns Ok(false),
    /// the streaming stops and the process is terminated.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    /// * `callback` - Function called for each line. Returns Ok(true) to continue, Ok(false) to stop.
    pub async fn execute_streaming_with_callback<F>(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        let full_args = self.build_args(args, port_override, baud_override)?;
        log::debug!("Executing (streaming): {:?} {:?}", self.bjig_path, full_args);

        let mut child = Command::new(self.bjig_path)
            .args(&full_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn bjig command: {}", e);
                e
            })?;

        let mut should_continue = true;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Some(line) = lines.next_line().await? {
                should_continue = callback(&line)?;
                if !should_continue {
                    // Kill the child process
                    log::debug!("Terminating child process");
                    let _ = child.kill().await;
                    break;
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().await?;

        // If we stopped intentionally, don't treat it as an error
        if !should_continue {
            log::debug!("Streaming stopped by callback");
            return Ok(());
        }

        if !status.success() {
            let stderr = if let Some(mut stderr) = child.stderr.take() {
                let mut buf = Vec::new();
                use tokio::io::AsyncReadExt;
                stderr.read_to_end(&mut buf).await?;
                String::from_utf8_lossy(&buf).to_string()
            } else {
                String::new()
            };

            log::error!("Streaming command failed - stderr: {}", stderr);

            return Err(BjigError::CommandFailed(format!(
                "Exit code: {:?}, stderr: {}",
                status.code(),
                stderr
            )));
        }

        Ok(())
    }

    /// Execute bjig command and stream stdout with external stop signal
    ///
    /// This variant allows external code to stop the streaming by sending
    /// a signal through the provided channel. Used for monitor commands
    /// that need to be controlled externally.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    /// * `stop_rx` - Receiver for stop signals
    pub async fn execute_streaming_with_stopper(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
        mut stop_rx: mpsc::Receiver<()>,
    ) -> Result<()> {
        let full_args = self.build_args(args, port_override, baud_override)?;
        log::debug!("Executing (streaming with stopper): {:?} {:?}", self.bjig_path, full_args);

        let mut child = Command::new(self.bjig_path)
            .args(&full_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn bjig command: {}", e);
                e
            })?;

        let mut stopped_externally = false;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            loop {
                tokio::select! {
                    // Line received from monitor
                    line_result = lines.next_line() => {
                        match line_result? {
                            Some(line) => {
                                println!("{}", line);
                            }
                            None => break,
                        }
                    }
                    // Stop signal received
                    _ = stop_rx.recv() => {
                        log::info!("Stop signal received, terminating monitor");
                        stopped_externally = true;
                        break;
                    }
                }
            }
        }

        // Kill the child process
        let _ = child.kill().await;
        let _ = child.wait().await;

        if stopped_externally {
            log::debug!("Streaming stopped by external signal");
        }

        Ok(())
    }

    /// Execute bjig command and stream stdout with control messages (stop/pause/resume)
    ///
    /// This variant allows external code to control the streaming with pause/resume/stop.
    /// When paused, data continues to be read but is not printed.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    /// * `control_rx` - Receiver for control messages
    pub async fn execute_streaming_with_control(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
        mut control_rx: mpsc::Receiver<ControlMessage>,
    ) -> Result<()> {
        let full_args = self.build_args(args, port_override, baud_override)?;
        log::debug!("Executing (streaming with control): {:?} {:?}", self.bjig_path, full_args);

        let mut child = Command::new(self.bjig_path)
            .args(&full_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn bjig command: {}", e);
                e
            })?;

        let mut paused = false;
        let mut stopped = false;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            loop {
                tokio::select! {
                    // Line received from monitor
                    line_result = lines.next_line() => {
                        match line_result? {
                            Some(line) => {
                                if !paused {
                                    println!("{}", line);
                                }
                                // If paused, data is discarded (router buffers it)
                            }
                            None => break,
                        }
                    }
                    // Control signal received
                    msg = control_rx.recv() => {
                        match msg {
                            Some(ControlMessage::Stop) => {
                                log::info!("Stop signal received, terminating monitor");
                                stopped = true;
                                break;
                            }
                            Some(ControlMessage::Pause) => {
                                log::info!("Pause signal received");
                                paused = true;
                            }
                            Some(ControlMessage::Resume) => {
                                log::info!("Resume signal received");
                                paused = false;
                            }
                            None => {
                                log::debug!("Control channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Kill the child process
        let _ = child.kill().await;
        let _ = child.wait().await;

        if stopped {
            log::debug!("Streaming stopped by control signal");
        }

        Ok(())
    }

    /// Execute bjig command and stream stdout with callback and external stop signal
    ///
    /// Combines callback functionality with external stop control.
    /// The callback is called for each line and can stop by returning Ok(false).
    /// External code can also stop via the stop channel.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    /// * `callback` - Function called for each line. Returns Ok(true) to continue, Ok(false) to stop.
    /// * `stop_rx` - Receiver for stop signals
    pub async fn execute_streaming_with_callback_and_stopper<F>(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
        mut callback: F,
        mut stop_rx: mpsc::Receiver<()>,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        let full_args = self.build_args(args, port_override, baud_override)?;
        log::debug!("Executing (streaming with callback and stopper): {:?} {:?}", self.bjig_path, full_args);

        let mut child = Command::new(self.bjig_path)
            .args(&full_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn bjig command: {}", e);
                e
            })?;

        let mut should_continue = true;
        let mut stopped_externally = false;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            loop {
                tokio::select! {
                    // Line received from monitor
                    line_result = lines.next_line() => {
                        let line_opt: Option<String> = line_result?;
                        match line_opt {
                            Some(line) => {
                                should_continue = callback(&line)?;
                                if !should_continue {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    // Stop signal received
                    _ = stop_rx.recv() => {
                        log::info!("Stop signal received, terminating monitor");
                        stopped_externally = true;
                        break;
                    }
                }
            }
        }

        // Kill the child process
        let _ = child.kill().await;
        let _ = child.wait().await;

        if stopped_externally {
            log::debug!("Streaming stopped by external signal");
        } else if !should_continue {
            log::debug!("Streaming stopped by callback");
        }

        Ok(())
    }

    /// Execute bjig command and stream stdout with callback and control messages
    ///
    /// Combines callback functionality with pause/resume/stop control.
    /// When paused, data continues to be read but callback is not invoked.
    ///
    /// # Arguments
    /// * `args` - Command arguments
    /// * `port_override` - Optional port override
    /// * `baud_override` - Optional baud override
    /// * `callback` - Function called for each line. Returns Ok(true) to continue, Ok(false) to stop.
    /// * `control_rx` - Receiver for control messages
    pub async fn execute_streaming_with_callback_and_control<F>(
        &self,
        args: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
        mut callback: F,
        mut control_rx: mpsc::Receiver<ControlMessage>,
    ) -> Result<()>
    where
        F: FnMut(&str) -> Result<bool>,
    {
        let full_args = self.build_args(args, port_override, baud_override)?;
        log::debug!("Executing (streaming with callback and control): {:?} {:?}", self.bjig_path, full_args);

        let mut child = Command::new(self.bjig_path)
            .args(&full_args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::error!("Failed to spawn bjig command: {}", e);
                e
            })?;

        let mut should_continue = true;
        let mut paused = false;
        let mut stopped = false;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            loop {
                tokio::select! {
                    // Line received from monitor
                    line_result = lines.next_line() => {
                        let line_opt: Option<String> = line_result?;
                        match line_opt {
                            Some(line) => {
                                if !paused {
                                    should_continue = callback(&line)?;
                                    if !should_continue {
                                        break;
                                    }
                                }
                                // If paused, data is discarded (router buffers it)
                            }
                            None => break,
                        }
                    }
                    // Control signal received
                    msg = control_rx.recv() => {
                        match msg {
                            Some(ControlMessage::Stop) => {
                                log::info!("Stop signal received, terminating monitor");
                                stopped = true;
                                break;
                            }
                            Some(ControlMessage::Pause) => {
                                log::info!("Pause signal received");
                                paused = true;
                            }
                            Some(ControlMessage::Resume) => {
                                log::info!("Resume signal received");
                                paused = false;
                            }
                            None => {
                                log::debug!("Control channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Kill the child process
        let _ = child.kill().await;
        let _ = child.wait().await;

        if stopped {
            log::debug!("Streaming stopped by control signal");
        } else if !should_continue {
            log::debug!("Streaming stopped by callback");
        }

        Ok(())
    }

    /// Build full command arguments with port and baud
    fn build_args(
        &self,
        subcommand: &[&str],
        port_override: Option<&str>,
        baud_override: Option<u32>,
    ) -> Result<Vec<String>> {
        let port = resolve_port(port_override, self.default_port)?;
        let baud = resolve_baud(baud_override, self.default_baud);

        let mut args = vec![
            "--port".to_string(),
            port,
            "--baud".to_string(),
            baud.to_string(),
        ];

        args.extend(subcommand.iter().map(|s| s.to_string()));

        Ok(args)
    }

    /// Run bjig command with given arguments
    async fn run_command(&self, args: &[String]) -> Result<String> {
        log::debug!("Executing: {:?} {:?}", self.bjig_path, args);

        let output = Command::new(self.bjig_path)
            .args(args)
            .output()
            .await
            .map_err(|e| {
                log::error!("Failed to execute bjig command: {}", e);
                e
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            log::error!("Command failed - stdout: {}, stderr: {}", stdout, stderr);

            return Err(BjigError::CommandFailed(format!(
                "Exit code: {:?}, stderr: {}",
                output.status.code(),
                stderr
            )));
        }

        let stdout = String::from_utf8(output.stdout)?;
        log::debug!("Command output: {}", stdout);

        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_args_with_overrides() {
        let executor = CommandExecutor::new(
            Path::new("/bin/bjig"),
            Some("/dev/ttyACM0"),
            Some(38400),
        );

        let args = executor
            .build_args(&["router", "get-version"], Some("/dev/ttyACM1"), Some(115200))
            .unwrap();

        assert_eq!(
            args,
            vec![
                "--port",
                "/dev/ttyACM1",
                "--baud",
                "115200",
                "router",
                "get-version"
            ]
        );
    }

    #[test]
    fn test_build_args_with_defaults() {
        let executor = CommandExecutor::new(
            Path::new("/bin/bjig"),
            Some("/dev/ttyACM0"),
            Some(38400),
        );

        let args = executor
            .build_args(&["router", "start"], None, None)
            .unwrap();

        assert_eq!(
            args,
            vec!["--port", "/dev/ttyACM0", "--baud", "38400", "router", "start"]
        );
    }
}
