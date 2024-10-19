use std::time::Duration;

use chrono::NaiveTime;
use duration_str::deserialize_duration;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum ControlConfig {
    /// Disabled control.
    #[default]
    Off,
    /// Cyclic activation and deactivation of the control pin.
    Cyclic {
        /// The GPIO pin used for control.
        pin: u32,
        /// The duration in seconds for which the control pin should
        /// be activated (0 means never).
        #[serde(deserialize_with = "deserialize_duration")]
        on_duration: Duration,
        /// The duration in seconds for which the control pin should
        /// be deactivated (0 means never).
        #[serde(deserialize_with = "deserialize_duration")]
        off_duration: Duration,
    },
    /// Activate and deactivate the control pin based on time stamps.
    TimeBased {
        /// The GPIO pin used for control.
        pin: u32,
        /// The time of the day when the control pin should be activated.
        activate_time: NaiveTime,
        /// The time of the day when the control pin should be deactivated.
        deactivate_time: NaiveTime,
    },
    /// Activate and deactivate the control pin based on measured values.
    Feedback {
        /// The GPIO pin used for control.
        pin: u32,
        /// The condition that activates the control pin.
        activate_condition: String,
        /// The condition that deactivates the control pin.
        deactivate_condition: String,
    },
}

impl ControlConfig {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Off)
    }
}
