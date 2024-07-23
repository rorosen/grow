use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AirConfig {
    pub control: ExhaustControlConfig,
    pub sample: AirSampleConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExhaustControlMode {
    /// Disabled exhaust fan control.
    Off,
    /// Cyclic exhaust fan control.
    Cyclic,
    /// Threshold exhaust fan control.
    Threshold,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExhaustControlConfig {
    /// The control mode.
    pub mode: ExhaustControlMode,
    /// The gpio pin used to control the exhaust fan.
    pub pin: u8,
    /// The duration in seconds for which the air fan should
    /// run (0 means never). Only has an effect in cyclic mode.
    pub on_duration_secs: u64,
    /// The duration in seconds for which the air fan should be
    /// stopped (0 means never). Only has an effect in cyclic mode.
    pub off_duration_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AirSampleConfig {
    /// The I2C address of the left air sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub left_address: u8,
    /// The I2C address of the right air sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub right_address: u8,
    /// The rate in which the air sensors take measurements in seconds.
    pub sample_rate_secs: u64,
}
