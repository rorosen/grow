use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WaterLevelConfig {
    pub control: PumpControlConfig,
    pub sample: WaterLevelSampleConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PumpControlConfig {
    /// Whether to enable pump control.
    pub enable: bool,
    /// The gpio pin used to control the right pump.
    pub left_pin: u8,
    /// The gpio pin used to control the right pump.
    pub right_pin: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaterLevelSampleConfig {
    /// The I2C address of the water level sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub sensor_address: u8,
    /// The rate in which the water level sensor takes measurements in seconds.
    pub sample_rate_secs: u64,
}
