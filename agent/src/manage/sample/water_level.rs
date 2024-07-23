use grow_measure::{water_level::WaterLevelSensor, WaterLevelMeasurement};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::WaterLevelSampleConfig;

pub struct WaterLevelSampler {
    sender: mpsc::Sender<WaterLevelMeasurement>,
    sensor_address: u8,
    sample_rate: Duration,
}

impl WaterLevelSampler {
    pub fn new(
        config: &WaterLevelSampleConfig,
        sender: mpsc::Sender<WaterLevelMeasurement>,
    ) -> Self {
        Self {
            sender,
            sensor_address: config.sensor_address,
            sample_rate: Duration::from_secs(config.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut sensor = WaterLevelSensor::new(self.sensor_address).await.ok();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if sensor.is_none() {
                        sensor = WaterLevelSensor::new(self.sensor_address).await.ok();
                    }

                    if let Some(s) = sensor.as_mut() {
                        match s.measure(cancel_token.clone()).await {
                            Ok(measurement) => {
                                self.sender
                                    .send(measurement)
                                    .await
                                    .expect("water level measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take water level measurement: {err}");
                                sensor = None;
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down water level sampler");
                    return;
                }
            }
        }
    }
}
