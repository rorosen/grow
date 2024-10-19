use std::{path::PathBuf, time::Duration};

use duration_str::deserialize_duration;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum SampleConfig {
    /// Disabled sampling.
    #[default]
    Off,
    /// Execute measurements in intervals with a fixed period of time in-between
    Interval {
        /// The period etween two intervals.
        #[serde(deserialize_with = "deserialize_duration")]
        period: Duration,
        /// The path to the python script that takes the measurememnt
        script_path: PathBuf,
    },
}

impl SampleConfig {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Off)
    }
}
