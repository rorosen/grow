use chrono::NaiveTime;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct LightConfig {
    #[serde(default)]
    pub control: LightControlConfig,
    #[serde(default)]
    pub sample: LightSampleConfig,
}

#[derive(PartialEq, Debug, Default, Deserialize)]
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

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct LightSampleConfig {
    /// The rate in which the light sensors take measurements in seconds.
    pub sample_rate_secs: u64,
    /// The light sensors in use.
    pub sensors: HashMap<String, LightSensorConfig>,
}

#[derive(PartialEq, Debug, Deserialize)]
pub struct LightSensorConfig {
    /// The type of the light sensor.
    pub model: LightSensorModel,
    /// The address of the light sensor.
    #[serde(deserialize_with = "super::from_hex")]
    pub address: u8,
}

#[derive(PartialEq, Debug, Deserialize)]
pub enum LightSensorModel {
    Bh1750Fvi,
}
