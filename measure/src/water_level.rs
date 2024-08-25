use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

pub mod vl53l0x;

#[async_trait]
pub trait WaterLevelSensor {
    async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<WaterLevelMeasurement, Error>;
}

/// A single water level measurement.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WaterLevelMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: Option<String>,
    /// The distance between the sensor and the water surface in mm.
    pub distance: u32,
}

impl WaterLevelMeasurement {
    pub fn new(measure_time: i64, distance: u32) -> Self {
        Self {
            measure_time,
            label: None,
            distance,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}
