use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::control::ControlConfig;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: AirSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirSampleConfig {
    /// The rate in which the air sensors are sampled in seconds.
    #[serde(default)]
    pub sample_rate_secs: u64,
    /// The air sensors in use.
    #[serde(default)]
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
