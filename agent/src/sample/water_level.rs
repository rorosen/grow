use anyhow::{bail, Context, Result};
use grow_measure::water_level::{vl53l0x::Vl53L0X, WaterLevelMeasurement, WaterLevelSensor};
use tracing::{debug, info, warn};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::{sync::mpsc, time::Interval};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::{WaterLevelSampleConfig, WaterLevelSensorModel};

pub struct WaterLevelSampler {
    sender: mpsc::Sender<Vec<WaterLevelMeasurement>>,
    interval: Interval,
    sensors: HashMap<String, Box<(dyn WaterLevelSensor + Send)>>,
}

impl WaterLevelSampler {
    pub async fn new(
        config: &WaterLevelSampleConfig,
        sender: mpsc::Sender<Vec<WaterLevelMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let sample_rate = Duration::from_secs(config.sample_rate_secs);
        if sample_rate.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        let mut sensors: HashMap<String, Box<dyn WaterLevelSensor + Send>> = HashMap::new();
        // Use async_iterator once stable: https://github.com/rust-lang/rust/issues/79024
        for (label, sensor_config) in &config.sensors {
            match sensor_config.model {
                WaterLevelSensorModel::Vl53L0X => {
                    let sensor = Vl53L0X::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!("Failed to initialize {label} water level sensor (Vl53L0X)",)
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
            info!("No water level sensors configured - water level sampler is disabled");
            return Ok(());
        }

        debug!("Starting water level sampler");
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
                                warn!("Failed to measure with {label} water level sensor: {err}");
                            }
                        };
                    }

                    if !measurements.is_empty() {
                        self.sender
                            .send(measurements)
                            .await
                            .context("Failed to send water level measurements")?;
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}
