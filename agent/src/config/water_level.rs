use std::collections::HashMap;

use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct WaterLevelConfig {
    #[serde(default)]
    pub control: PumpControlConfig,
    #[serde(default)]
    pub sample: WaterLevelSampleConfig,
}

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct PumpControlConfig {
    /// Whether to enable pump control.
    pub enable: bool,
    /// The water pumps in use.
    pub pumps: HashMap<String, u8>,
}

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct WaterLevelSampleConfig {
    /// The rate in which the water level sensor takes measurements in seconds.
    pub sample_rate_secs: u64,
    /// The water level sensors in use.
    pub sensors: HashMap<String, WaterLevelSensorConfig>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct WaterLevelSensorConfig {
    /// The model of the water level sensor.
    pub model: WaterLevelSensorModel,
    /// The address of the water level sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub address: u8,
}

#[derive(PartialEq, Debug, Deserialize)]
pub enum WaterLevelSensorModel {
    Vl53Lox,
}
