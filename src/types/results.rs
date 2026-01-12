//! Result types for bjig command responses

use serde::{Deserialize, Serialize};

/// Router start result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResult {
    pub result: String,
    pub message: String,
}

impl StartResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Router stop result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopResult {
    pub result: String,
    pub message: String,
}

impl StopResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Router firmware version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub version: String,
}

/// Scan mode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMode {
    pub mode: u8,
    pub mode_name: String,
}

/// Set scan mode result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetScanModeResult {
    pub result: String,
    pub message: String,
}

impl SetScanModeResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Module ID list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIdList {
    pub module_count: usize,
    pub modules: Vec<String>,
}

/// Remove module ID result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveResult {
    pub result: String,
    pub message: String,
}

impl RemoveResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Keep alive result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepAliveResult {
    pub result: String,
    pub message: String,
}

impl KeepAliveResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// DFU (firmware update) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfuResult {
    pub result: String,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl DfuResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// DFU progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DfuProgress {
    pub phase: String,
    pub chunk_number: usize,
    pub total_chunks: usize,
    pub percentage: u8,
}

/// Set parameter result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterResult {
    pub result: String,
    pub message: String,
}

impl SetParameterResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Module restart result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartResult {
    pub result: String,
    pub message: String,
}

impl RestartResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}

/// Module control command result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    pub result: String,
    pub message: String,
}

impl ControlResult {
    pub fn is_success(&self) -> bool {
        self.result == "success"
    }
}
