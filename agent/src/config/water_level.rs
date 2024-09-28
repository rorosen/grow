use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::control::ControlConfig;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: WaterLevelSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelSampleConfig {
    /// The rate in which the water level sensor takes measurements in seconds.
    #[serde(default)]
    pub sample_rate_secs: u64,
    /// The water level sensors in use.
    #[serde(default)]
    pub sensors: HashMap<String, WaterLevelSensorConfig>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct WaterLevelSensorConfig {
    /// The model of the water level sensor.
    pub model: WaterLevelSensorModel,
    /// The address of the water level sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub address: u8,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum WaterLevelSensorModel {
    Vl53L0X,
}
