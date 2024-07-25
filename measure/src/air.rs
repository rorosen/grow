use std::time::Duration;

use grow_hardware::i2c::I2C;
use params::Params;
use sensor_data::SensorData;
use tokio_util::sync::CancellationToken;

use crate::{AirMeasurement, Error};

mod params;
mod sensor_data;

const OVERSAMPLING_X2: u8 = 0b010;
#[allow(dead_code)]
const OVERSAMPLING_X4: u8 = 0b011;
#[allow(dead_code)]
const OVERSAMPLING_X8: u8 = 0b100;
const OVERSAMPLING_X16: u8 = 0b101;

const CMD_SOFT_RESET: u8 = 0xB6;
const CHIP_ID: u8 = 0x61;
const MODE_SLEEP: u8 = 0x00;
const MODE_FORCED: u8 = 0x01;
const MODE_MASK: u8 = 0x03;

const PARAMS_SIZE1: usize = 23;
const PARAMS_SIZE2: usize = 14;
const PARAMS_SIZE3: usize = 5;
const DATA_SIZE: usize = 17;

const REG_PARAMS1: u8 = 0x8A;
const REG_PARAMS2: u8 = 0xE1;
const REG_PARAMS3: u8 = 0x00;
const REG_CHIP_ID: u8 = 0xD0;
const REG_RESET: u8 = 0xE0;
const REG_CTRL_MEAS: u8 = 0x74;
const REG_CTRL_HUM: u8 = 0x72;
const REG_GAS_WAIT0: u8 = 0x64;
const REG_RES_HEAT0: u8 = 0x5A;
const REG_CTRL_GAS1: u8 = 0x71;
const REG_DATA0: u8 = 0x1D;

/// BME680
pub struct AirSensor {
    i2c: I2C,
    params: Option<Params>,
}

impl AirSensor {
    pub async fn new(address: u8) -> Result<Self, Error> {
        let mut i2c = I2C::new(address).await?;
        let params = match Self::init_params(&mut i2c).await {
            Ok(params) => Some(params),
            Err(err) => {
                log::warn!("failed to initialize BME680 at address 0x{address:02x}: {err}");
                None
            }
        };

        Ok(Self { i2c, params })
    }

    pub async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<AirMeasurement, Error> {
        if self.params.is_none() {
            self.params = Self::init_params(&mut self.i2c).await.ok();
        }

        self.set_op_mode(MODE_SLEEP).await?;
        self.ensure_oversampling(OVERSAMPLING_X2, OVERSAMPLING_X2, OVERSAMPLING_X16)
            .await?;
        self.set_heater_config(25, 300, 700).await?;
        self.set_op_mode(MODE_FORCED).await?;
        let data = self.read_sensor_data(cancel_token).await?;
        let params = self.params.as_ref().ok_or(Error::NotInit("air (BME680)"))?;
        let (t_fine, temperature) = params.calc_temperature(data.temp_adc);

        Ok(AirMeasurement {
            temperature,
            humidity: params.calc_humidity(data.hum_adc, temperature),
            pressure: params.calc_pressure(data.press_adc, t_fine) / 100.,
            resistance: params.compute_resistance(data.gas_adc, data.gas_range as usize),
        })
    }

    async fn read_sensor_data(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<SensorData, Error> {
        let mut buf = [0; DATA_SIZE];
        self.i2c.read_reg_bytes(REG_DATA0, &mut buf).await?;

        if (buf[0] & 0x80) > 0 {
            return Ok(SensorData::new(&buf));
        }

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(Error::Cancelled);
                },
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    self.i2c.read_reg_bytes(REG_DATA0, &mut buf).await?;
                    if (buf[0] & 0x80) > 0 {
                        return Ok(SensorData::new(&buf));
                    }

                }
            }
        }
    }

    async fn set_op_mode(&mut self, mode: u8) -> Result<(), Error> {
        let ctr_meas = self.i2c.read_reg_byte(REG_CTRL_MEAS).await?;
        self.i2c
            .write_reg_byte(REG_CTRL_MEAS, (ctr_meas & !MODE_MASK) | mode)
            .await?;

        Ok(())
    }

    async fn ensure_oversampling(
        &mut self,
        humidity: u8,
        temperature: u8,
        pressure: u8,
    ) -> Result<(), Error> {
        const OSRS_HMASK: u8 = 0x07;
        const OSRS_TMASK: u8 = 0xE0;
        const OSRS_PMASK: u8 = 0x1C;
        const OSRS_TP_MASK: u8 = OSRS_TMASK | OSRS_PMASK;

        let ctr_hum = self.i2c.read_reg_byte(REG_CTRL_HUM).await?;
        if (ctr_hum & OSRS_HMASK) != humidity {
            self.i2c
                .write_reg_byte(REG_CTRL_HUM, (ctr_hum & !OSRS_HMASK) | humidity)
                .await?;
        }

        let ctr_meas = self.i2c.read_reg_byte(REG_CTRL_MEAS).await?;
        let desired_ctr_meas = (temperature << 5) | (pressure << 2);
        if (ctr_meas & OSRS_TP_MASK) != desired_ctr_meas {
            self.i2c
                .write_reg_byte(REG_CTRL_MEAS, (ctr_meas & !OSRS_TP_MASK) | desired_ctr_meas)
                .await?;
        }

        Ok(())
    }

    async fn set_heater_config(
        &mut self,
        ambient_temperature: i8,
        temperature: u16,
        duration: u16,
    ) -> Result<(), Error> {
        let Some(ref params) = self.params else {
            return Err(Error::NotInit("BME680"));
        };

        self.i2c
            .write_reg_byte(
                REG_RES_HEAT0,
                params.calc_heat_resistance(ambient_temperature, temperature),
            )
            .await?;

        self.i2c
            .write_reg_byte(REG_GAS_WAIT0, params.calc_gas_wait(duration))
            .await?;

        // enable run gas and select heater profile 0
        self.i2c.write_reg_byte(REG_CTRL_GAS1, 1 << 4).await?;

        Ok(())
    }

    async fn init_params(i2c: &mut I2C) -> Result<Params, Error> {
        let id = i2c.read_reg_byte(REG_CHIP_ID).await?;

        if id != CHIP_ID {
            return Err(Error::IdentifyFailed(String::from("air (BME680)")));
        }

        i2c.write_reg_byte(REG_RESET, CMD_SOFT_RESET).await?;

        tokio::time::sleep(Duration::from_millis(5)).await;

        let mut params = [0; PARAMS_SIZE1 + PARAMS_SIZE2 + PARAMS_SIZE3];
        let (mut params1, rest) = params.split_at_mut(PARAMS_SIZE1);
        let (mut params2, mut params3) = rest.split_at_mut(PARAMS_SIZE2);

        i2c.read_reg_bytes(REG_PARAMS1, &mut params1).await?;
        i2c.read_reg_bytes(REG_PARAMS2, &mut params2).await?;
        i2c.read_reg_bytes(REG_PARAMS3, &mut params3).await?;

        Ok(Params::new(&params))
    }
}
