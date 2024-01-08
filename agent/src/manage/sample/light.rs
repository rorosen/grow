#![allow(dead_code)]

use crate::{error::AppError, i2c::I2C};
use api::gen::grow::{LightMeasurement, LightMeasurements};
use clap::Parser;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::parse_hex_u8;

const MODE_ONE_TIME_HIGH_RES: u8 = 0x20;
const WAIT_DURATION: Duration = Duration::from_millis(200);
const MT_REG_MAX: u8 = 31;
const MT_REG_DEFAULT: u8 = 69;
const MASK_MT_REG_MIN: u8 = 0x1F;
const CMD_SET_MT_HIGH: u8 = 0b01000 << 3;
const CMD_SET_MT_LOW: u8 = 0b011 << 5;

/// BH1750FVI
pub struct LightSensor {
    i2c: I2C,
}

impl LightSensor {
    pub async fn new(address: u8) -> Result<Self, AppError> {
        let i2c = I2C::new(address).await?;

        Ok(Self { i2c })
    }

    pub async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<LightMeasurement, AppError> {
        self.i2c
            .write_bytes(&[CMD_SET_MT_HIGH | (MT_REG_MAX >> 5)])
            .await?;

        self.i2c
            .write_bytes(&[CMD_SET_MT_LOW | (MT_REG_MAX & MASK_MT_REG_MIN)])
            .await?;

        self.i2c.write_bytes(&[MODE_ONE_TIME_HIGH_RES]).await?;

        tokio::select! {
            _ = cancel_token.cancelled() => {
                log::debug!("aborting light measurement: token cancelled");
                return Err(AppError::Cancelled);
            }
            _ = tokio::time::sleep(WAIT_DURATION) => {
                let mut buf = [0; 2];
                self.i2c.read_bytes(&mut buf[..]).await?;
                return Ok(LightMeasurement {
                    lux: ((((buf[0] as u32) << 8) | (buf[1] as u32)) as f64) / 1.2 * ((MT_REG_DEFAULT as f64) / (MT_REG_MAX as f64))
                })
            }
        }
    }
}

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

pub struct LightSampler {
    sender: mpsc::Sender<LightMeasurements>,
    left_sensor: LightSensor,
    right_sensor: LightSensor,
    sample_rate: Duration,
}

impl LightSampler {
    pub async fn new(
        args: &LightSampleArgs,
        sender: mpsc::Sender<LightMeasurements>,
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

                    self.sender
                        .send(LightMeasurements{
                            measure_time: Some(SystemTime::now().into()),
                            left: left_measurement,
                            right: right_measurement})
                        .await
                        .expect("light measurements channel is open");
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("shutting down light sampler");
                    return;
                }
            }
        }
    }
}
