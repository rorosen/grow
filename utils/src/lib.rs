use std::time::SystemTime;

use api::grow::{AirMeasurement, AirSample, LightMeasurement, LightSample, WaterLevelMeasurement};
use bson::DateTime;
use serde::{Deserialize, Serialize};

pub mod api;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageAirMeasurement {
    measure_time: DateTime,
    left: Option<AirSample>,
    right: Option<AirSample>,
}

impl TryFrom<AirMeasurement> for StorageAirMeasurement {
    type Error = String;

    fn try_from(value: AirMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let t = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(t),
            left: value.left,
            right: value.right,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLightMeasurement {
    measure_time: DateTime,
    left: Option<LightSample>,
    right: Option<LightSample>,
}

impl TryFrom<LightMeasurement> for StorageLightMeasurement {
    type Error = String;

    fn try_from(value: LightMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let t = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(t),
            left: value.left,
            right: value.right,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageWaterLevelMeasurement {
    measure_time: DateTime,
    distance: u32,
}

impl TryFrom<WaterLevelMeasurement> for StorageWaterLevelMeasurement {
    type Error = String;

    fn try_from(value: WaterLevelMeasurement) -> Result<Self, Self::Error> {
        let Some(time) = value.measure_time else {
            return Err(String::from("message has no measure time"));
        };

        let t = match SystemTime::try_from(time) {
            Ok(time) => time,
            Err(err) => {
                return Err(format!(
                    "failed to convert measure time to system time: {err}"
                ))
            }
        };

        Ok(Self {
            measure_time: DateTime::from_system_time(t),
            distance: value.distance,
        })
    }
}
