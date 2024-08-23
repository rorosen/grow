use anyhow::{Context, Result};
use grow_measure::water_level::{vl53l0x::Vl53L0X, WaterLevelMeasurement, WaterLevelSensor};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::{WaterLevelSampleConfig, WaterLevelSensorModel};

pub struct WaterLevelSample {
    pub measure_time: SystemTime,
    pub measurements: HashMap<String, WaterLevelMeasurement>,
}

pub struct WaterLevelSampler {
    sender: mpsc::Sender<WaterLevelSample>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn WaterLevelSensor + Send)>>,
}

impl WaterLevelSampler {
    pub async fn new(
        config: &WaterLevelSampleConfig,
        sender: mpsc::Sender<WaterLevelSample>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn WaterLevelSensor + Send>> = HashMap::new();

        for (identifier, sensor_config) in &config.sensors {
            match sensor_config.model {
                WaterLevelSensorModel::Vl53L0x => {
                    let sensor = Vl53L0X::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to initialize {identifier} water level sensor (Vl53L0X)",
                            )
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

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("Starting water level sampler");

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
                                log::warn!("Failed to measure with {identifier} water level sensor: {err}");
                            }
                        };
                    }

                    let sample = WaterLevelSample{
                        measure_time: SystemTime::now(),
                        measurements,
                    };

                    self.sender
                        .send(sample)
                        .await
                        .expect("Water level measurements channel is open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("Stopping water level sampler");
                    return;
                }
            }
        }
    }
}
