use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirPumpConfig {
    #[serde(default)]
    pub control: AirPumpControlConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum AirPumpControlConfig {
    /// Disabled air pump control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the control pin.
    Cyclic {
        /// The GPIO pin used to control the air pump.
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
        /// The GPIO pin used to control the air pump.
        pin: u32,
        /// The time of the day when the control pin should be activated.
        activate_time: NaiveTime,
        /// The time of the day when the control pin should be deactivated.
        deactivate_time: NaiveTime,
    },
}
