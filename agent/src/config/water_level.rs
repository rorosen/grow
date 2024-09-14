use std::collections::HashMap;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelConfig {
    #[serde(default)]
    pub control: WaterLevelControlConfig,
    #[serde(default)]
    pub sample: WaterLevelSampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum WaterLevelControlConfig {
    /// Disabled water level control.
    #[default]
    Off,
    /// Activate and deactivate all water pumps together based on time stamps.
    TimeBased {
        /// The time of the day when the water pumps should be switched on.
        activate_time: NaiveTime,
        /// The time of the day when the water pumps should be switched off.
        deactivate_time: NaiveTime,
        /// The identifier and corresponding control pin of a water pump.
        pumps: HashMap<String, u32>,
    },
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
    Vl53L0X,
}
