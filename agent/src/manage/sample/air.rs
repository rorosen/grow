use clap::Parser;
use common::AirMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;

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

    /// The rate in which the air sensor takes measurements in seconds
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
    args: AirSampleArgs,
}

impl AirSampler {
    pub fn new(args: AirSampleArgs, sender: mpsc::Sender<AirMeasurement>) -> Self {
        Self { sender, args }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<(), AppError> {
        let mut left_sensor = AirSensor::new(self.args.left_address).await.ok();
        let mut right_sensor = AirSensor::new(self.args.right_address).await.ok();

        loop {}

        Ok(())
    }
}
