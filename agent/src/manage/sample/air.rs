use anyhow::{Context, Result};
use grow_measure::{air::AirSensor, AirMeasurement};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::air::AirSampleConfig;

pub struct AirSample {
    pub measure_time: SystemTime,
    pub left: Option<AirMeasurement>,
    pub right: Option<AirMeasurement>,
}

pub struct AirSampler {
    sender: mpsc::Sender<AirSample>,
    left_sensor: AirSensor,
    right_sensor: AirSensor,
    sample_rate: Duration,
}

impl AirSampler {
    pub async fn new(config: &AirSampleConfig, sender: mpsc::Sender<AirSample>) -> Result<Self> {
        let left_sensor = AirSensor::new(config.left_address)
            .await
            .context("failed to initialize left air sensor")?;
        let right_sensor = AirSensor::new(config.right_address)
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
        log::debug!("starting air sampler");
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    let left_measurement = match self.left_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::trace!("could not take left air measurement: {err}");
                            None
                        }
                    };

                    let right_measurement = match self.right_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::trace!("could not take right air measurement: {err}");
                            None
                        }
                    };

                    if left_measurement.is_some() || right_measurement.is_some() {
                        let sample = AirSample {
                            measure_time: SystemTime::now(),
                            left: left_measurement,
                            right: right_measurement,
                        };

                        self.sender
                            .send(sample)
                            .await
                            .expect("air measurements channel is open");
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping air sampler");
                    return;
                }
            }
        }
    }
}
