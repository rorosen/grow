use std::time::Duration;

use clap::Parser;
use grow_measure::{water_level::WaterLevelSensor, WaterLevelMeasurement};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::parse_hex_u8;

#[derive(Debug, Parser)]
pub struct WaterLevelSampleArgs {
    /// The I2C address of the water_level sensor
    #[arg(
        id = "water_level_sample_sensor_address",
        long = "water_level-sample-right-sensor-address",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_RIGHT_SENSOR_ADDRESS",
        value_parser=parse_hex_u8,
        default_value = "0x28"
    )]
    sensor_address: u8,

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
    sender: mpsc::Sender<WaterLevelMeasurement>,
    sensor_address: u8,
    sample_rate: Duration,
}

impl WaterLevelSampler {
    pub fn new(args: &WaterLevelSampleArgs, sender: mpsc::Sender<WaterLevelMeasurement>) -> Self {
        Self {
            sender,
            sensor_address: args.sensor_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut sensor = WaterLevelSensor::new(self.sensor_address).await.ok();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if sensor.is_none() {
                        sensor = WaterLevelSensor::new(self.sensor_address).await.ok();
                    }

                    if let Some(s) = sensor.as_mut() {
                        match s.measure(cancel_token.clone()).await {
                            Ok(measurement) => {
                                self.sender
                                    .send(measurement)
                                    .await
                                    .expect("water level measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take water level measurement: {err}");
                                sensor = None;
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
