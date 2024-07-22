use anyhow::{Context, Result};
use clap::Parser;
use grow_measure::{air::AirSensor, AirMeasurement};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::parse_hex_u8;

#[derive(Debug, Parser)]
pub struct AirSampleArgs {
    /// The I2C address of the left air sensor
    #[arg(
        id = "air_sample_left_sensor_address",
        long = "air-sample-left-sensor-address",
        env = "GROW_AGENT_AIR_SAMPLE_LEFT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x76"
    )]
    left_address: u8,

    /// The I2C address of the right air sensor
    #[arg(
        id = "air_sample_right_sensor_address",
        long = "air-sample-right-sensor-address",
        env = "GROW_AGENT_AIR_SAMPLE_RIGHT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x77"
    )]
    right_address: u8,

    /// The rate in which the air sensors take measurements in seconds
    #[arg(
        id = "air_sample_rate_secs",
        long = "air-sample-rate-secs",
        env = "GROW_AGENT_AIR_SAMPLE_RATE_SECS",
        default_value_t = 300
    )]
    sample_rate_secs: u64,
}

pub struct AirSample {
    pub measure_time: SystemTime,
    pub left: Option<AirMeasurement>,
    pub right: Option<AirMeasurement>,
}

pub struct AirSampler {
    sender: mpsc::Sender<AirSample>,
    left_sensor: AirSensor,
    right_sensor: AirSensor,
    sample_rate: Duration,
}

impl AirSampler {
    pub async fn new(args: &AirSampleArgs, sender: mpsc::Sender<AirSample>) -> Result<Self> {
        let left_sensor = AirSensor::new(args.left_address)
            .await
            .context("failed to initialize left air sensor")?;
        let right_sensor = AirSensor::new(args.right_address)
            .await
            .context("failed to initialize right air sensor")?;

        Ok(Self {
            sender,
            left_sensor,
            right_sensor,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    let left_measurement = match self.left_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::warn!("could not take left air measurement: {err}");
                            None
                        }
                    };

                    let right_measurement = match self.right_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::warn!("could not take right air measurement: {err}");
                            None
                        }
                    };

                    if left_measurement.is_some() || right_measurement.is_some() {
                        let sample = AirSample {
                            measure_time: SystemTime::now(),
                            left: left_measurement,
                            right: right_measurement,
                        };

                        self.sender
                            .send(sample)
                            .await
                            .expect("air measurements channel is open");
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down air sampler");
                    return;
                }
            }
        }
    }
}
