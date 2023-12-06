#![allow(dead_code)]

use crate::{error::AppError, i2c::I2C};
use clap::Parser;
use common::LightMeasurement;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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
                return Ok(LightMeasurement {lux: ((((buf[0] as u32) << 8) | (buf[1] as u32)) as f64) / 1.2 * ((MT_REG_DEFAULT as f64) / (MT_REG_MAX as f64))})
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
        default_value_t = 0x5C
    )]
    left_address: u8,

    /// The I2C address of the right light sensor
    #[arg(
        id = "light_sample_right_sensor_address",
        long = "light-sample-right-sensor-address",
        env = "GROW_AGENT_LIGHT_SAMPLE_RIGHT_SENSOR_ADDRESS",
        default_value_t = 0x23
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
    sender: mpsc::Sender<(&'static str, LightMeasurement)>,
    left_address: u8,
    right_address: u8,
    sample_rate: Duration,
}

impl LightSampler {
    pub fn new(
        args: &LightSampleArgs,
        sender: mpsc::Sender<(&'static str, LightMeasurement)>,
    ) -> Self {
        Self {
            sender,
            left_address: args.left_address,
            right_address: args.right_address,
            sample_rate: Duration::from_secs(args.sample_rate_secs),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        let mut left_sensor = LightSensor::new(self.left_address).await.ok();
        let mut right_sensor = LightSensor::new(self.right_address).await.ok();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.sample_rate) => {
                    if left_sensor.is_none() {
                        left_sensor = LightSensor::new(self.left_address).await.ok();
                    }

                    if right_sensor.is_none() {
                        right_sensor = LightSensor::new(self.right_address).await.ok();
                    }

                    if let Some(sensor) = left_sensor.as_mut() {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(light_measurement) => {
                                self.sender
                                    .send(("left", light_measurement))
                                    .await
                                    .expect("light measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take left light measurement: {err}");
                                left_sensor = LightSensor::new(self.left_address).await.ok();
                            }
                        }
                    }

                    if let Some(sensor) = right_sensor.as_mut() {
                        match sensor.measure(cancel_token.clone()).await {
                            Ok(light_measurement) => {
                                self.sender
                                    .send(("right", light_measurement))
                                    .await
                                    .expect("light measurement channel is open");
                            }
                            Err(err) => {
                                log::warn!("could not take right light measurement: {err}");
                                right_sensor = LightSensor::new(self.right_address).await.ok();
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::info!("shutting down light sampler");
                    return;
                }
            }
        }
    }
}
