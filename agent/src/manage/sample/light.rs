use std::time::{Duration, SystemTime};

use clap::Parser;
use grow_measure::{light::LightSensor, LightMeasurement};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;

use super::parse_hex_u8;

#[derive(Debug, Parser)]
pub struct LightSampleArgs {
    /// The I2C address of the left light sensor
    #[arg(
        id = "light_sample_left_sensor_address",
        long = "light-sample-left-sensor-address",
        env = "GROW_AGENT_LIGHT_SAMPLE_LEFT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x5C"
    )]
    left_address: u8,

    /// The I2C address of the right light sensor
    #[arg(
        id = "light_sample_right_sensor_address",
        long = "light-sample-right-sensor-address",
        env = "GROW_AGENT_LIGHT_SAMPLE_RIGHT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x23"
    )]
    right_address: u8,

    /// The rate in which the light sensors take measurements in seconds
    #[arg(
        id = "light_sample_rate_secs",
        long = "light-sample-rate-secs",
        env = "GROW_AGENT_LIGHT_SAMPLE_RATE_SECS",
        default_value_t = 300
    )]
    sample_rate_secs: u64,
}

pub struct LightSample {
    pub measure_time: SystemTime,
    pub left: Option<LightMeasurement>,
    pub right: Option<LightMeasurement>,
}

pub struct LightSampler {
    sender: mpsc::Sender<LightSample>,
    left_sensor: LightSensor,
    right_sensor: LightSensor,
    sample_rate: Duration,
}

impl LightSampler {
    pub async fn new(
        args: &LightSampleArgs,
        sender: mpsc::Sender<LightSample>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            sender,
            left_sensor: LightSensor::new(args.left_address).await?,
            right_sensor: LightSensor::new(args.right_address).await?,
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
                            log::warn!("could not take left light measurement: {err}");
                            None
                        }
                    };

                    let right_measurement = match self.right_sensor.measure(cancel_token.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::warn!("could not take right light measurement: {err}");
                            None
                        }
                    };

                    if left_measurement.is_some() || right_measurement.is_some() {
                        let sample = LightSample {
                            measure_time: SystemTime::now(),
                            left: left_measurement,
                            right: right_measurement,
                        };

                        self.sender
                            .send(sample)
                            .await
                            .expect("light measurements channel is open");
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down light sampler");
                    return;
                }
            }
        }
    }
}
