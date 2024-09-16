use anyhow::{bail, Context, Result};
use grow_measure::light::{bh1750fvi::Bh1750Fvi, LightMeasurement, LightSensor};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::{sync::mpsc, time::Interval};
use tokio_util::sync::CancellationToken;

use crate::config::light::{LightSampleConfig, LightSensorModel};

pub struct LightSampler {
    sender: mpsc::Sender<Vec<LightMeasurement>>,
    interval: Interval,
    sensors: HashMap<String, Box<(dyn LightSensor + Send)>>,
}

impl LightSampler {
    pub async fn new(
        config: &LightSampleConfig,
        sender: mpsc::Sender<Vec<LightMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let sample_rate = Duration::from_secs(config.sample_rate_secs);
        if sample_rate.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        let mut sensors: HashMap<String, Box<dyn LightSensor + Send>> = HashMap::new();
        // Use async_iterator once stable: https://github.com/rust-lang/rust/issues/79024
        for (label, sensor_config) in &config.sensors {
            match sensor_config.model {
                LightSensorModel::Bh1750Fvi => {
                    let sensor = Bh1750Fvi::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!("Failed to initilaize {label} light sensor (BH1750FVI)",)
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

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Light sampler";

        if self.sensors.is_empty() {
            log::info!("No light sensors configured - light sampler is disabled");
            return Ok(IDENTIFIER);
        }

        log::debug!("Starting light sampler");
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
                                log::warn!("Failed to measure with {label} light sensor: {err}");
                            }
                        };
                    }

                    if !measurements.is_empty() {
                        self.sender
                            .send(measurements)
                            .await
                            .context("Failed to send light measurements")?;
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(IDENTIFIER);
                }
            }
        }
    }
}
