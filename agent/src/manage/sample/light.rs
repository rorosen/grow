use anyhow::{Context, Result};
use grow_measure::{light::LightSensor, LightMeasurement};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::light::LightSampleConfig;

pub struct LightSample {
    pub measure_time: SystemTime,
    pub left: Option<LightMeasurement>,
    pub right: Option<LightMeasurement>,
}

pub struct LightSampler {
    sender: mpsc::Sender<LightSample>,
    left_sensor: LightSensor,
    right_sensor: LightSensor,
    sample_rate: Duration,
}

impl LightSampler {
    pub async fn new(
        config: &LightSampleConfig,
        sender: mpsc::Sender<LightSample>,
    ) -> Result<Self> {
        let left_sensor = LightSensor::new(config.left_address)
            .await
            .context("failed to initialize left light sensor")?;
        let right_sensor = LightSensor::new(config.right_address)
            .await
            .context("failed to initialize right air sensor")?;

        Ok(Self {
            sender,
            left_sensor,
            right_sensor,
            sample_rate: Duration::from_secs(config.sample_rate_secs),
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("starting light sampler");
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    let left_measurement = match self.left_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::trace!("could not take left light measurement: {err}");
                            None
                        }
                    };

                    let right_measurement = match self.right_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::trace!("could not take right light measurement: {err}");
                            None
                        }
                    };

                    if left_measurement.is_some() || right_measurement.is_some() {
                        let sample = LightSample {
                            measure_time: SystemTime::now(),
                            left: left_measurement,
                            right: right_measurement,
                        };

                        self.sender
                            .send(sample)
                            .await
                            .expect("light measurements channel is open");
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping light sampler");
                    return;
                }
            }
        }
    }
}
