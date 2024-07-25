use anyhow::{Context, Result};
use grow_measure::{water_level::WaterLevelSensor, WaterLevelMeasurement};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::WaterLevelSampleConfig;

pub struct WaterLevelSampler {
    sender: mpsc::Sender<WaterLevelMeasurement>,
    sensor: WaterLevelSensor,
    sample_rate: Duration,
}

impl WaterLevelSampler {
    pub async fn new(
        config: &WaterLevelSampleConfig,
        sender: mpsc::Sender<WaterLevelMeasurement>,
    ) -> Result<Self> {
        let sensor = WaterLevelSensor::new(config.sensor_address)
            .await
            .context("failed to initialize water level sensor")?;

        Ok(Self {
            sender,
            sensor,
            sample_rate: Duration::from_secs(config.sample_rate_secs),
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("starting water level sampler");

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    match self.sensor.measure(cancel_token.clone()).await {
                        Ok(measurement) => {
                            self.sender
                                .send(measurement)
                                .await
                                .expect("water level measurement channel is open");
                        }
                        Err(err) => log::trace!("could not take water level measurement: {err}"),
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping water level sampler");
                    return;
                }
            }
        }
    }
}
