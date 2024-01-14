use std::time::Duration;

use clap::Parser;
use grow_utils::api::grow::WaterLevelMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use self::sensor::WaterLevelSensor;

use super::parse_hex_u8;

mod sensor;

#[derive(Debug, Parser)]
pub struct WaterLevelSampleArgs {
    /// The I2C address of the left water_level sensor
    #[arg(
        id = "water_level_sample_left_sensor_address",
        long = "water_level-sample-left-sensor-address",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_LEFT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x29"
    )]
    left_address: u8,

    /// The I2C address of the right water_level sensor
    #[arg(
        id = "water_level_sample_right_sensor_address",
        long = "water_level-sample-right-sensor-address",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_RIGHT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x28"
    )]
    right_address: u8,

    /// The rate in which the water_level sensors take measurements in seconds
    #[arg(
        id = "water_level_sample_rate_secs",
        long = "water_level-sample-rate-secs",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_RATE_SECS",
        default_value_t = 300
    )]
    sample_rate_secs: u64,
}

pub struct WaterLevelSampler {
    sender: mpsc::Sender<(&'static str, WaterLevelMeasurement)>,
    left_address: u8,
    right_address: u8,
    sample_rate: Duration,
}

impl WaterLevelSampler {
    pub fn new(
        args: &WaterLevelSampleArgs,
        sender: mpsc::Sender<(&'static str, WaterLevelMeasurement)>,
    ) -> Self {
        Self {
            sender,
            left_address: args.left_address,
            right_address: args.right_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut left_sensor = WaterLevelSensor::new(self.left_address).await.ok();
        let mut right_sensor = WaterLevelSensor::new(self.right_address).await.ok();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if left_sensor.is_none() {
                        left_sensor = WaterLevelSensor::new(self.left_address).await.ok();
                    }

                    if right_sensor.is_none() {
                        right_sensor = WaterLevelSensor::new(self.right_address).await.ok();
                    }

                    if let Some(sensor) = left_sensor.as_mut() {
                        match sensor.measure().await {
                            Ok(light_measurement) => {
                                self.sender
                                    .send(("left", light_measurement))
                                    .await
                                    .expect("water level measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take left water level measurement: {err}");
                                left_sensor = WaterLevelSensor::new(self.left_address).await.ok();
                            }
                        }
                    }

                    if let Some(sensor) = right_sensor.as_mut() {
                        match sensor.measure().await {
                            Ok(light_measurement) => {
                                self.sender
                                    .send(("right", light_measurement))
                                    .await
                                    .expect("water level measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take right water level measurement: {err}");
                                right_sensor = WaterLevelSensor::new(self.right_address).await.ok();
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down water level sampler");
                    return;
                }
            }
        }
    }
}
