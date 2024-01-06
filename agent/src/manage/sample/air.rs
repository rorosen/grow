use std::time::Duration;

use clap::Parser;
use common::AirMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use self::air_sensor::AirSensor;

use super::parse_hex_u8;

mod air_sensor;
mod params;
mod sensor_data;
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

pub struct AirMeasurements(pub Option<AirMeasurement>, pub Option<AirMeasurement>);

pub struct AirSampler {
    sender: mpsc::Sender<AirMeasurements>,
    left_address: u8,
    right_address: u8,
    sample_rate: Duration,
}

impl AirSampler {
    pub fn new(args: &AirSampleArgs, sender: mpsc::Sender<AirMeasurements>) -> Self {
        Self {
            sender,
            left_address: args.left_address,
            right_address: args.right_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut left_sensor = AirSensor::new_opt(self.left_address).await;
        let mut right_sensor = AirSensor::new_opt(self.right_address).await;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if left_sensor.is_none() {
                        left_sensor = AirSensor::new_opt(self.left_address).await;
                    }

                    if right_sensor.is_none() {
                        right_sensor = AirSensor::new_opt(self.right_address).await;
                    }

                    let left_measurement = match left_sensor.as_mut() {
                        Some(s) => match s.measure(cancel_token.clone()).await {
                            Ok(m) => Some(m),
                            Err(err) => {
                                log::warn!("could not take left air measurement: {err}");
                                left_sensor = None;
                                None
                            }
                        },
                        None => None,
                    };

                    let right_measurement = match right_sensor.as_mut() {
                        Some(s) => match s.measure(cancel_token.clone()).await {
                            Ok(m) => Some(m),
                            Err(err) => {
                                log::warn!("could not take right air measurement: {err}");
                                right_sensor = None;
                                None
                            }
                        },
                        None => None,
                    };

                    self.sender
                        .send(AirMeasurements(left_measurement, right_measurement))
                        .await
                        .expect("air measurements channel is open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down air sampler");
                    return;
                }
            }
        }
    }
}
