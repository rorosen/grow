use anyhow::{Context, Result};
use grow_measure::{
    air::{bme680::Bme680, AirSensor},
    AirMeasurement,
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::config::air::{AirSampleConfig, AirSensorModel};

pub struct AirSample {
    pub measure_time: SystemTime,
    pub measurements: HashMap<String, AirMeasurement>,
}

pub struct AirSampler {
    sender: mpsc::Sender<AirSample>,
    sample_rate: Duration,
    sensors: HashMap<String, Box<(dyn AirSensor)>>,
}

impl AirSampler {
    pub async fn new(config: &AirSampleConfig, sender: mpsc::Sender<AirSample>) -> Result<Self> {
        let mut sensors: HashMap<String, Box<dyn AirSensor>> = HashMap::new();

        // Use async_iterator once stable: https://github.com/rust-lang/rust/issues/79024
        for (identifier, sensor_config) in &config.sensors {
            match sensor_config.model {
                AirSensorModel::Bme680 => {
                    let sensor = Bme680::new(sensor_config.address).await.with_context(|| {
                        format!(
                            "Failed to initialize air sensor (BME680) with identifier {identifier}",
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
        log::debug!("starting air sampler");
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
                                log::warn!("Failed to measure air with sensor {identifier}: {err}");
                            }
                        };
                    }

                    let sample = AirSample {
                        measure_time: SystemTime::now(),
                        measurements,
                    };

                    self.sender
                        .send(sample)
                        .await
                        .expect("air measurements channel is open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping air sampler");
                    return;
                }
            }
        }
    }
}
