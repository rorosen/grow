use anyhow::{Context, Result};
use futures::future::join_all;
use tokio_util::sync::CancellationToken;

use crate::measure::Measure;

pub struct Sampler<M: Measure> {
    sensors: Vec<M>,
}

impl<M> Sampler<M>
where
    M: Measure,
    M::Measurement: Send + Sync + 'static,
{
    pub fn new(sensors: Vec<M>) -> Self {
        Self { sensors }
    }

    pub async fn take_measurements(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<Vec<M::Measurement>> {
        let measurements = join_all(
            self.sensors
                .iter_mut()
                .map(|s| Self::measure(s, cancel_token.clone())),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<M::Measurement>>>()?;

        Ok(measurements)
    }

    async fn measure(sensor: &mut M, cancel_token: CancellationToken) -> Result<M::Measurement> {
        sensor
            .measure(cancel_token)
            .await
            .with_context(|| format!("Failed to measure with {:?} sensor", sensor.label()))
    }
}
