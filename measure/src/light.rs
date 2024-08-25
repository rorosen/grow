use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

pub mod bh1750fvi;

#[async_trait]
pub trait LightSensor {
    async fn measure(&mut self, cancel_token: CancellationToken)
        -> Result<LightMeasurement, Error>;
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
/// A single light measurement.
pub struct LightMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: Option<String>,
    /// The illuminance in lux.
    pub illuminance: f64,
}

impl LightMeasurement {
    pub fn new(measure_time: i64, illuminance: f64) -> Self {
        Self {
            measure_time,
            label: None,
            illuminance,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}
