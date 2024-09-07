use crate::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio_util::sync::CancellationToken;

pub mod bh1750fvi;

#[async_trait]
pub trait LightSensor {
    async fn measure(
        &mut self,
        label: String,
        cancel_token: CancellationToken,
    ) -> Result<LightMeasurement, Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
/// A single light measurement.
pub struct LightMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: String,
    /// The illuminance in lux.
    pub illuminance: Option<f64>,
}

impl LightMeasurement {
    pub fn new(measure_time: i64, label: String) -> Self {
        Self {
            measure_time,
            label,
            illuminance: None,
        }
    }

    pub fn illuminance(mut self, illuminance: f64) -> Self {
        self.illuminance = Some(illuminance);
        self
    }
}
