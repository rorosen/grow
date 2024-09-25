use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::control::ControlConfig;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct LightConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: LightSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct LightSampleConfig {
    /// The rate in which the light sensors take measurements in seconds.
    pub sample_rate_secs: u64,
    /// The light sensors in use.
    pub sensors: HashMap<String, LightSensorConfig>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct LightSensorConfig {
    /// The type of the light sensor.
    pub model: LightSensorModel,
    /// The address of the light sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub address: u8,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum LightSensorModel {
    Bh1750Fvi,
}
