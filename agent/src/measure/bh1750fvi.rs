use anyhow::{bail, Result};
use chrono::Utc;
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::measure::LightMeasurement;

use super::{i2c::I2C, Measure};

const MODE_ONE_TIME_HIGH_RES: u8 = 0x20;
const WAIT_DURATION: Duration = Duration::from_millis(200);
const MT_REG_MAX: u8 = 31;
const MT_REG_DEFAULT: u8 = 69;
const MASK_MT_REG_MIN: u8 = 0x1F;
const CMD_SET_MT_HIGH: u8 = 0b01000 << 3;
const CMD_SET_MT_LOW: u8 = 0b011 << 5;

/// BH1750FVI
pub struct Bh1750Fvi {
    i2c: I2C,
    label: String,
}

impl Bh1750Fvi {
    pub async fn new(i2c_path: impl AsRef<Path>, address: u8, label: String) -> Result<Self> {
        let i2c = I2C::new(i2c_path, address).await?;

        Ok(Self { i2c, label })
    }
}

impl Measure for Bh1750Fvi {
    type Measurement = LightMeasurement;

    async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<Self::Measurement> {
        self.i2c
            .write_bytes(&[CMD_SET_MT_HIGH | (MT_REG_MAX >> 5)])
            .await?;

        self.i2c
            .write_bytes(&[CMD_SET_MT_LOW | (MT_REG_MAX & MASK_MT_REG_MIN)])
            .await?;

        self.i2c.write_bytes(&[MODE_ONE_TIME_HIGH_RES]).await?;
        let measure_time = Utc::now().timestamp();

        tokio::select! {
            _ = cancel_token.cancelled() => {
                bail!("Measurement cancelled");
            }
            _ = tokio::time::sleep(WAIT_DURATION) => {
                let mut buf = [0; 2];
                self.i2c.read_bytes(&mut buf[..]).await?;
                let illuminance = ((((buf[0] as u32) << 8) | (buf[1] as u32)) as f64) / 1.2 * ((MT_REG_DEFAULT as f64) / (MT_REG_MAX as f64));
                let measurement = LightMeasurement::new(measure_time, self.label.clone()).illuminance(illuminance);

                Ok(measurement)
            }
        }
    }

    fn label(&self) ->  &str {
        &self.label
    }
}
