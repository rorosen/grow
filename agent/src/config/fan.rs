use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct FanConfig {
    #[serde(default)]
    pub control: FanControlConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum FanControlConfig {
    /// Disabled fan control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the fan control pin.
    Cyclic {
        pin: u32,
        /// The duration in seconds for which the fan control pin should be
        /// activated (0 means never).
        on_duration_secs: u64,
        /// The duration in seconds for which the fan control pin should be
        /// deactivated (0 means never).
        off_duration_secs: u64,
    },
}
