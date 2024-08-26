use anyhow::{Context, Result};
use grow_measure::air::{bme680::Bme680, AirMeasurement, AirSensor};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::air::{AirSampleConfig, AirSensorModel};

pub struct AirSampler {
    sender: mpsc::Sender<Vec<AirMeasurement>>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn AirSensor + Send)>>,
}

impl AirSampler {
    pub async fn new(
        config: &AirSampleConfig,
        sender: mpsc::Sender<Vec<AirMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn AirSensor + Send>> = HashMap::new();

        // Use async_iterator once stable: https://github.com/rust-lang/rust/issues/79024
        for (label, sensor_config) in &config.sensors {
            match sensor_config.model {
                AirSensorModel::Bme680 => {
                    let sensor = Bme680::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!("Failed to initialize {label} air sensor (BME680)",)
                        })?;
                    sensors.insert(label.into(), Box::new(sensor));
                }
            }
        }

        Ok(Self {
            sender,
            sample_rate: Duration::from_secs(config.sample_rate_secs),
            sensors,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Air sampler";

        if self.sensors.is_empty() {
            log::info!("No air sensors configured - air sampler is disabled");
            return Ok(IDENTIFIER);
        }

        log::info!("Starting air sampler");
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    let mut measurements = vec![];

                    for (label, sensor) in &mut self.sensors {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(measurement) => {
                                measurements.push(measurement.label(label.into()));
                            },
                            Err(err) => {
                                log::warn!("Failed to measure with {label} air sensor: {err}");
                            }
                        };
                    }

                    self.sender
                        .send(measurements)
                        .await
                        .context("Failed to send air measurements")?;
                }
                _ = cancel_token.cancelled() => {
                    return Ok(IDENTIFIER);
                }
            }
        }
    }
}
