use std::time::Duration;

use clap::Parser;
use common::WaterLevelMeasurement;
use tokio::sync::mpsc;

mod sensor;

#[derive(Debug, Parser)]
pub struct WaterLevelSampleArgs {
    /// The I2C address of the left water_level sensor
    #[arg(
        id = "water_level_sample_left_sensor_address",
        long = "water_level-sample-left-sensor-address",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_LEFT_SENSOR_ADDRESS",
        default_value_t = 0x76
    )]
    left_address: u8,

    /// The I2C address of the right water_level sensor
    #[arg(
        id = "water_level_sample_right_sensor_address",
        long = "water_level-sample-right-sensor-address",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_RIGHT_SENSOR_ADDRESS",
        default_value_t = 0x77
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
    sender: mpsc::Sender<WaterLevelMeasurement>,
    left_address: u8,
    right_address: u8,
    sample_rate: Duration,
}

impl WaterLevelSampler {
    pub fn new(args: &WaterLevelSampleArgs, sender: mpsc::Sender<WaterLevelMeasurement>) -> Self {
        Self {
            sender,
            left_address: args.left_address,
            right_address: args.right_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }
}
