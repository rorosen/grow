use crate::{Error, WaterLevelMeasurement};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

pub mod vl53l0x;

#[async_trait]
pub trait WaterLevelSensor {
    async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<WaterLevelMeasurement, Error>;
}
