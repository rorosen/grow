use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum FanControlMode {
    /// Disabled fan control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the fan control pin.
    Cyclic,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct FanControlConfig {
    /// The control mode.
    pub mode: FanControlMode,
    #[serde(default)]
    pub pin: u32,
    /// The duration in seconds for which the fan control pin should be
    /// activated (0 means never). Only has an effect in cyclic control mode.
    #[serde(default)]
    pub on_duration_secs: u64,
    /// The duration in seconds for which the fan control pin should be
    /// deactivated (0 means never). Only has an effect in cyclic control mode.
    #[serde(default)]
    pub off_duration_secs: u64,
}
