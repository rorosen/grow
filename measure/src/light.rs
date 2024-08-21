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
    /// The illuminance in lux.
    pub illuminance: f64,
}
