use anyhow::{Context, Result};
use grow_measure::water_level::{vl53l0x::Vl53L0X, WaterLevelMeasurement, WaterLevelSensor};
use std::{collections::HashMap, path::Path, time::Duration};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::{WaterLevelSampleConfig, WaterLevelSensorModel};

pub struct WaterLevelSampler {
    sender: mpsc::Sender<Vec<WaterLevelMeasurement>>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn WaterLevelSensor + Send)>>,
}

impl WaterLevelSampler {
    pub async fn new(
        config: &WaterLevelSampleConfig,
        sender: mpsc::Sender<Vec<WaterLevelMeasurement>>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn WaterLevelSensor + Send>> = HashMap::new();

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
            sample_rate: Duration::from_secs(config.sample_rate_secs),
            sensors,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Water level sampler";

        if self.sensors.is_empty() {
            log::info!("No water level sensors configured - water level sampler is disabled");
            return Ok(IDENTIFIER);
        }

        log::info!("Starting water level sampler");
        self.sample_and_send(&cancel_token).await?;
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    self.sample_and_send(&cancel_token).await?;
                }
                _ = cancel_token.cancelled() => {
                    return Ok(IDENTIFIER);
                }
            }
        }
    }

    async fn sample_and_send(&mut self, cancel_token: &CancellationToken) -> Result<()> {
        let mut measurements = vec![];

        for (label, sensor) in &mut self.sensors {
            match sensor.measure(label.into(), cancel_token.clone()).await {
                Ok(measurement) => {
                    measurements.push(measurement);
                }
                Err(err) => {
                    log::warn!("Failed to measure with {label} water level sensor: {err}");
                }
            };
        }

        if !measurements.is_empty() {
            self.sender
                .send(measurements)
                .await
                .context("Failed to send water level measurements")?;
        }

        Ok(())
    }
}
