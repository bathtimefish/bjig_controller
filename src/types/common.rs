//! Common types shared across the library

use serde::{Deserialize, Serialize};

/// Sensor information from get-supported-sensor-id command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
    pub sensor_id: String,
    pub sensor_name: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Module configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub module_id: String,
    pub sensor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Scan mode enum for type-safe API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanModeType {
    LongRange,
    Legacy,
}

impl ScanModeType {
    /// Convert to u8 value for bjig command
    pub fn to_u8(self) -> u8 {
        match self {
            ScanModeType::LongRange => 0,
            ScanModeType::Legacy => 1,
        }
    }

    /// Parse from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ScanModeType::LongRange),
            1 => Some(ScanModeType::Legacy),
            _ => None,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ScanModeType::LongRange => "LongRange",
            ScanModeType::Legacy => "Legacy",
        }
    }
}
