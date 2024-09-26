use anyhow::{bail, Context, Result};
use grow_measure::air::{bme680::Bme680, AirMeasurement, AirSensor};
use tracing::{debug, info, warn};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::{sync::mpsc, time::Interval};
use tokio_util::sync::CancellationToken;

use crate::config::air::{AirSampleConfig, AirSensorModel};

pub struct AirSampler {
    sender: mpsc::Sender<Vec<AirMeasurement>>,
    interval: Interval,
    sensors: HashMap<String, Box<(dyn AirSensor + Send)>>,
}

impl AirSampler {
    pub async fn new(
        config: &AirSampleConfig,
        sender: mpsc::Sender<Vec<AirMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let sample_rate = Duration::from_secs(config.sample_rate_secs);
        if sample_rate.is_zero() {
            bail!("Sample rate cannot be zero");
        }

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
            interval: tokio::time::interval(sample_rate),
            sensors,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        if self.sensors.is_empty() {
            info!("No air sensors configured - air sampler is disabled");
            return Ok(());
        }

        debug!("Starting air sampler");
        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    let mut measurements = vec![];

                    for (label, sensor) in &mut self.sensors {
                        match sensor.measure(label.into(), cancel_token.clone()).await {
                            Ok(measurement) => {
                                measurements.push(measurement);
                            }
                            Err(err) => {
                                warn!("Failed to measure with {label} air sensor: {err}");
                            }
                        };
                    }

                    if !measurements.is_empty() {
                        self.sender
                            .send(measurements)
                            .await
                            .context("Failed to send air measurements")?;
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}
