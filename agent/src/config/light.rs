use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct LightConfig {
    #[serde(default)]
    pub control: LightControlConfig,
    #[serde(default)]
    pub sample: LightSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum LightControlConfig {
    /// Disabled light control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the control pin.
    Cyclic {
        /// The GPIO pin used to control the light.
        pin: u32,
        /// The duration in seconds for which the control pin should be
        /// activated (0 means never).
        on_duration_secs: u64,
        /// The duration in seconds for which the control pin should be
        /// deactivated (0 means never).
        off_duration_secs: u64,
    },
    /// Activate and deactivate the control pin based on time stamps.
    TimeBased {
        /// The GPIO pin used to control the light.
        pin: u32,
        /// The time of the day when the control pin should be switched on.
        activate_time: NaiveTime,
        /// The time of the day when the control pin should be switched off.
        deactivate_time: NaiveTime,
    },
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
