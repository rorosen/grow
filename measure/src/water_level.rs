use crate::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio_util::sync::CancellationToken;

pub mod vl53l0x;

#[async_trait]
pub trait WaterLevelSensor {
    async fn measure(
        &mut self,
        label: String,
        cancel_token: CancellationToken,
    ) -> Result<WaterLevelMeasurement, Error>;
}

/// A single water level measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct WaterLevelMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: String,
    /// The distance between the sensor and the water surface in mm.
    pub distance: Option<u32>,
}

impl WaterLevelMeasurement {
    pub fn new(measure_time: i64, label: String) -> Self {
        Self {
            measure_time,
            label,
            distance: None,
        }
    }

    pub fn distance(mut self, distance: u32) -> Self {
        self.distance = Some(distance);
        self
    }
}
