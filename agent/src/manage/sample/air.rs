use std::time::Duration;

use clap::Parser;
use common::AirMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use self::air_sensor::AirSensor;

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
        default_value_t = 0x76
    )]
    left_address: u8,

    /// The I2C address of the right air sensor
    #[arg(
        id = "air_sample_right_sensor_address",
        long = "air-sample-right-sensor-address",
        env = "GROW_AGENT_AIR_SAMPLE_RIGHT_SENSOR_ADDRESS",
        default_value_t = 0x77
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

pub struct AirSampler {
    sender: mpsc::Sender<AirMeasurement>,
    left_address: u8,
    right_address: u8,
    sample_rate: Duration,
}

impl AirSampler {
    pub fn new(args: &AirSampleArgs, sender: mpsc::Sender<AirMeasurement>) -> Self {
        Self {
            sender,
            left_address: args.left_address,
            right_address: args.right_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut left_sensor = AirSensor::new(self.left_address).await.ok();
        let mut right_sensor = AirSensor::new(self.right_address).await.ok();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if left_sensor.is_none() {
                        left_sensor = AirSensor::new(self.left_address).await.ok();
                    }

                    if right_sensor.is_none() {
                        right_sensor = AirSensor::new(self.right_address).await.ok();
                    }

                    if let Some(sensor) = left_sensor.as_mut() {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(air_measurement) => {
                                self.sender
                                    .send(air_measurement)
                                    .await
                                    .expect("air measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take left air measurement: {err}");
                                left_sensor = AirSensor::new(self.left_address).await.ok();
                            }
                        }
                    }

                    if let Some(sensor) = right_sensor.as_mut() {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(air_measurement) => {
                                self.sender
                                    .send(air_measurement)
                                    .await
                                    .expect("air measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take right air measurement: {err}");
                                right_sensor = AirSensor::new(self.right_address).await.ok();
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::info!("shutting down air sampler");
                    return;
                }
            }
        }
    }
}
