use crate::{Error, LightMeasurement};
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

pub mod bh1750fvi;

#[async_trait]
pub trait LightSensor {
    async fn measure(&mut self, cancel_token: CancellationToken)
        -> Result<LightMeasurement, Error>;
}
