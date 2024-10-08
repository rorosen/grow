use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use futures::future::join_all;
use tokio::{
    sync::broadcast,
    time::interval,
};
use tokio_util::sync::CancellationToken;

use crate::measure::Measure;

pub struct Sampler<M: Measure> {
    period: Duration,
    sender: broadcast::Sender<Vec<M::Measurement>>,
    sensors: Vec<M>,
}

impl<M> Sampler<M>
where
    M: Measure,
    M::Measurement: Send + Sync + 'static,
{
    pub fn new(
        sample_rate_secs: u64,
        sender: broadcast::Sender<Vec<M::Measurement>>,
        sensors: Vec<M>,
    ) -> Result<Self> {
        let period = Duration::from_secs(sample_rate_secs);
        if !sensors.is_empty() && period.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        Ok(Self {
            sender,
            sensors,
            period,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        if self.sensors.is_empty() {
            return Ok(());
        }

        let mut interval = interval(self.period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let measurements = join_all(
                        self.sensors
                            .iter_mut()
                            .map(|s| Self::measure(s, cancel_token.clone())),
                    )
                    .await
                    .into_iter()
                    .collect::<Result<Vec<M::Measurement>>>()
                    .context("Failed to take measurements")?;

                    self.sender
                        .send(measurements)
                        .map_err(|_| anyhow!("channel closed"))
                        .context("Failed to send measurements")?;
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }

    async fn measure(sensor: &mut M, cancel_token: CancellationToken) -> Result<M::Measurement> {
        sensor
            .measure(cancel_token)
            .await
            .with_context(|| format!("Failed to measure with {:?} sensor", sensor.label()))
    }
}
