use crate::{AirMeasurement, Error};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

pub mod bme680;

#[async_trait]
pub trait AirSensor {
    async fn measure(&mut self, cancel_token: CancellationToken) -> Result<AirMeasurement, Error>;
}
