use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirConfig {
    #[serde(default)]
    pub control: ExhaustControlConfig,
    #[serde(default)]
    pub sample: AirSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ExhaustControlMode {
    /// Disabled exhaust fan control.
    #[default]
    Off,
    /// Cyclic exhaust fan control.
    Cyclic,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ExhaustControlConfig {
    /// The control mode.
    pub mode: ExhaustControlMode,
    /// The gpio pin used to control the exhaust fan.
    pub pin: u32,
    /// The duration in seconds for which the exhaust fan control pin should
    /// be activated (0 means never). Only has an effect in cyclic mode.
    pub on_duration_secs: u64,
    /// The duration in seconds for which the exhaust fan control pin should
    /// be deactivated (0 means never). Only has an effect in cyclic mode.
    pub off_duration_secs: u64,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirSampleConfig {
    /// The rate in which the air sensors are sampled in seconds.
    pub sample_rate_secs: u64,
    /// The air sensors in use.
    pub sensors: HashMap<String, AirSensorConfig>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct AirSensorConfig {
    /// The type of the air sensor.
    pub model: AirSensorModel,
    /// The address of the air sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub address: u8,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum AirSensorModel {
    Bme680,
}
