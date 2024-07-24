use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize)]
pub struct WaterLevelConfig {
    pub control: PumpControlConfig,
    pub sample: WaterLevelSampleConfig,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct PumpControlConfig {
    /// Whether to enable pump control.
    pub enable: bool,
    /// The gpio pin used to control the pump.
    pub pin: u8,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct WaterLevelSampleConfig {
    /// The I2C address of the water level sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub sensor_address: u8,
    /// The rate in which the water level sensor takes measurements in seconds.
    pub sample_rate_secs: u64,
}
