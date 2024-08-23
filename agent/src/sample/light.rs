use anyhow::{Context, Result};
use grow_measure::light::{bh1750fvi::Bh1750Fvi, LightMeasurement, LightSensor};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::light::{LightSampleConfig, LightSensorModel};

pub struct LightSample {
    pub measure_time: SystemTime,
    pub measurements: HashMap<String, LightMeasurement>,
}

pub struct LightSampler {
    sender: mpsc::Sender<LightSample>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn LightSensor + Send)>>,
}

impl LightSampler {
    pub async fn new(
        config: &LightSampleConfig,
        sender: mpsc::Sender<LightSample>,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn LightSensor + Send>> = HashMap::new();
        for (identifier, sensor_config) in &config.sensors {
            match sensor_config.model {
                LightSensorModel::Bh1750Fvi => {
                    let sensor = Bh1750Fvi::new(&i2c_path, sensor_config.address)
                        .await
                        .with_context(|| {
                            format!("Failed to initilaize {identifier} light sensor (BH1750FVI)",)
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
        log::debug!("Starting light sampler");
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
                                log::warn!("Failed to measure with {identifier} light sensor: {err}");
                            }
                        };
                    }

                    let sample = LightSample {
                        measure_time: SystemTime::now(),
                        measurements,
                    };

                    self.sender
                        .send(sample)
                        .await
                        .expect("Light measurements channel should be open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("Stopping light sampler");
                    return;
                }
            }
        }
    }
}
