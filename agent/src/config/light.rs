use chrono::NaiveTime;
use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize)]
pub struct LightConfig {
    pub control: LightControlConfig,
    pub sample: LightSampleConfig,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct LightControlConfig {
    /// Whether to enable time based light control.
    pub enable: bool,
    /// The gpio pin used to control the light.
    pub pin: u8,
    /// The time of the day when the light should be switched on.
    /// Only has an effect if control is enabled.
    pub activate_time: NaiveTime,
    /// The time of the day when the light should be switched off.
    /// Only has an effect if control is enabled.
    pub deactivate_time: NaiveTime,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct LightSampleConfig {
    /// The I2C address of the left light sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub left_address: u8,
    /// The I2C address of the right light sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub right_address: u8,
    /// The rate in which the light sensors take measurements in seconds.
    pub sample_rate_secs: u64,
}
