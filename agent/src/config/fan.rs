use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct FanControlConfig {
    /// Whether to enable cyclic fan control
    #[serde(default)]
    pub enable: bool,
    /// The gpio pin used to control the circulation fans
    #[serde(default)]
    pub pin: u8,
    /// The duration in seconds for which the circulation fans should
    /// run (0 means never). Only has an effect if control is enabled.
    #[serde(default)]
    pub on_duration_secs: u64,
    /// The duration in seconds for which the circulation fans should be
    /// stopped (0 means never). Only has an effect if control is enabled.
    #[serde(default)]
    pub off_duration_secs: u64,
}
