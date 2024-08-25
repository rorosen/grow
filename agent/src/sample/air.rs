use anyhow::{Context, Result};
use grow_measure::air::{bme680::Bme680, AirMeasurement, AirSensor};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::air::{AirSampleConfig, AirSensorModel};

pub struct AirSampler {
    sender: mpsc::Sender<HashMap<String, AirMeasurement>>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn AirSensor + Send)>>,
}

impl AirSampler {
    pub async fn new(
        config: &AirSampleConfig,
        sender: mpsc::Sender<HashMap<String, AirMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn AirSensor + Send>> = HashMap::new();

        // Use async_iterator once stable: https://github.com/rust-lang/rust/issues/79024
        for (identifier, sensor_config) in &config.sensors {
            match sensor_config.model {
                AirSensorModel::Bme680 => {
                    let sensor = Bme680::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!("Failed to initialize {identifier} air sensor (BME680)",)
                        })?;
                    sensors.insert(identifier.into(), Box::new(sensor));
                }
            }
        }

        Ok(Self {
            sender,
            sample_rate: Duration::from_secs(config.sample_rate_secs),
            sensors,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        log::debug!("Starting air sampler");
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    let mut measurements = HashMap::new();

                    for (identifier, sensor) in &mut self.sensors {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(measurement) => {
                                measurements.insert(identifier.into(), measurement);
                            },
                            Err(err) => {
                                log::warn!("Failed to measure with {identifier} air sensor: {err}");
                            }
                        };
                    }

                    self.sender
                        .send(measurements)
                        .await
                        .expect("Air measurements channel should be open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("Stopping air sampler");
                    return Ok(());
                }
            }
        }
    }
}
