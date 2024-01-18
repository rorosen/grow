use std::time::SystemTime;

use api::grow::{AirMeasurement, AirSample, LightMeasurement, WaterLevelMeasurement};
use bson::DateTime;
use serde::{Deserialize, Serialize};

pub mod api;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageAirMeasurement {
    pub measure_time: DateTime,
    pub left: Option<AirSample>,
    pub right: Option<AirSample>,
}

impl TryFrom<AirMeasurement> for StorageAirMeasurement {
    type Error = String;

    fn try_from(value: AirMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let time = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(time),
            left: value.left,
            right: value.right,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLightMeasurement {
    pub measure_time: DateTime,
    pub left: f64,
    pub right: f64,
}

impl TryFrom<LightMeasurement> for StorageLightMeasurement {
    type Error = String;

    fn try_from(value: LightMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let time = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(time),
            left: value.left,
            right: value.right,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageWaterLevelMeasurement {
    pub measure_time: DateTime,
    pub distance: u32,
}

impl TryFrom<WaterLevelMeasurement> for StorageWaterLevelMeasurement {
    type Error = String;

    fn try_from(value: WaterLevelMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let time = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(time),
            distance: value.distance,
        })
    }
}
