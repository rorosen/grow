use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelConfig {
    #[serde(default)]
    pub control: WaterLevelControlConfig,
    #[serde(default)]
    pub sample: WaterLevelSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum WaterLevelControlMode {
    /// Disabled water level control.
    #[default]
    Off,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelControlConfig {
    /// The control mode.
    pub mode: WaterLevelControlMode,
    /// The water pumps in use.
    pub pumps: HashMap<String, u32>,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelSampleConfig {
    /// The rate in which the water level sensor takes measurements in seconds.
    pub sample_rate_secs: u64,
    /// The water level sensors in use.
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
    Vl53L0x,
}
