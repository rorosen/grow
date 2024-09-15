use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirConfig {
    #[serde(default)]
    pub control: AirControlConfig,
    #[serde(default)]
    pub sample: AirSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum AirControlConfig {
    /// Disabled exhaust fan control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the control pin.
    Cyclic {
        /// The GPIO pin used to control the air quality, e.g. via an exhaust fan.
        pin: u32,
        /// The duration in seconds for which the control pin should
        /// be activated (0 means never).
        on_duration_secs: u64,
        /// The duration in seconds for which the control pin should
        /// be deactivated (0 means never).
        off_duration_secs: u64,
    },
    /// Activate and deactivate the control pin based on time stamps.
    TimeBased {
        /// The GPIO pin used to control the air quality, e.g. via an exhaust fan.
        pin: u32,
        /// The time of the day when the control pin should be activated.
        activate_time: NaiveTime,
        /// The time of the day when the control pin should be deactivated.
        deactivate_time: NaiveTime,
    },
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirSampleConfig {
    /// The rate in which the air sensors are sampled in seconds.
    pub sample_rate_secs: u64,
    /// The air sensors in use.
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
