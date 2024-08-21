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
    /// The distance in mm.
    pub distance: u32,
}
