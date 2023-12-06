use common::AirMeasurement;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::{error::AppError, i2c::I2C};

use super::{params::Params, sensor_data::SensorData};

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
const TEMPERATURE_MAX: u16 = 400;
const GAS_WAIT_MS_MAX: u16 = 4032;
const GAS_WAIT_VALUE_MAX: u8 = 0xFF;

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
    params: Params,
}

impl AirSensor {
    pub async fn new(address: u8) -> Result<Self, AppError> {
        let mut i2c = I2C::new(address).await?;
        let id = i2c.read_reg_byte(REG_CHIP_ID).await?;

        if id != CHIP_ID {
            return Err(AppError::IdentifyFailed(String::from("air (BME680)")));
        }

        i2c.write_reg_byte(REG_RESET, CMD_SOFT_RESET).await?;

        tokio::time::sleep(Duration::from_millis(5)).await;

        let mut params = [0; PARAMS_SIZE1 + PARAMS_SIZE2 + PARAMS_SIZE3];
        let (mut params1, rest) = params.split_at_mut(PARAMS_SIZE1);
        let (mut params2, mut params3) = rest.split_at_mut(PARAMS_SIZE2);

        i2c.read_reg_bytes(REG_PARAMS1, &mut params1).await?;

        i2c.read_reg_bytes(REG_PARAMS2, &mut params2).await?;

        i2c.read_reg_bytes(REG_PARAMS3, &mut params3).await?;

        Ok(Self {
            i2c,
            params: Params::new(&params),
        })
    }

    pub async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<AirMeasurement, AppError> {
        self.set_op_mode(MODE_SLEEP).await?;
        self.ensure_oversampling(OVERSAMPLING_X2, OVERSAMPLING_X2, OVERSAMPLING_X16)
            .await?;
        self.set_heater_config(25, 300, 700).await?;
        self.set_op_mode(MODE_FORCED).await?;
        let data = self.read_sensor_data(cancel_token).await?;
        let (t_fine, temperature) = self.calc_temperature(data.temp_adc);

        Ok(AirMeasurement {
            temperature,
            humidity: self.calc_humidity(data.hum_adc, temperature),
            pressure: self.calc_pressure(data.press_adc, t_fine) / 100.,
            resistance: self.compute_resistance(data.gas_adc, data.gas_range as usize),
        })
    }

    async fn read_sensor_data(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<SensorData, AppError> {
        let mut buf = [0; DATA_SIZE];
        self.i2c.read_reg_bytes(REG_DATA0, &mut buf).await?;

        if (buf[0] & 0x80) > 0 {
            return Ok(SensorData::new(&buf));
        }

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(AppError::Cancelled);
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

    async fn set_op_mode(&mut self, mode: u8) -> Result<(), AppError> {
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
    ) -> Result<(), AppError> {
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
    ) -> Result<(), AppError> {
        self.i2c
            .write_reg_byte(
                REG_RES_HEAT0,
                self.calc_heat_resistance(ambient_temperature, temperature),
            )
            .await?;

        self.i2c
            .write_reg_byte(REG_GAS_WAIT0, self.calc_gas_wait(duration))
            .await?;

        // enable run gas and select heater profile 0
        self.i2c.write_reg_byte(REG_CTRL_GAS1, 1 << 4).await?;

        Ok(())
    }

    fn calc_heat_resistance(&self, ambient_temperature: i8, mut temperature: u16) -> u8 {
        if temperature > TEMPERATURE_MAX {
            temperature = TEMPERATURE_MAX;
        }

        let var1 = ((self.params.gas.gh1 as f64) / 16.) + 49.;
        let var2 = (((self.params.gas.gh2 as f64) / 32768.) * 0.00005) + 0.00235;
        let var3 = (self.params.gas.gh3 as f64) / 1024.;
        let var4 = var1 * (1. + (var2 * (temperature as f64)));
        let var5 = var4 + (var3 * (ambient_temperature as f64));
        (3.4 * (var5
            * (4.
                / (4.
                    + (self.params.gas.heat_range as f64)
                        * (1. / 1. + (self.params.gas.heat_val as f64) * 0.002)))
            - 25.)) as u8
    }

    fn calc_gas_wait(&self, mut duration: u16) -> u8 {
        if duration > GAS_WAIT_MS_MAX {
            return GAS_WAIT_VALUE_MAX;
        }

        let mut factor: u8 = 0;
        while duration > 0x3F {
            duration /= 4;
            factor += 1;
        }

        (duration as u8) + (factor * 64)
    }

    fn calc_temperature(&self, temp_adc: u32) -> (f64, f64) {
        let var1 = (((temp_adc as f64) / 16384.) - ((self.params.temp.t1 as f64) / 1024.))
            * (self.params.temp.t2 as f64);
        let var2 = ((((temp_adc as f64) / 131072.) - ((self.params.temp.t1 as f64) / 8192.))
            * (((temp_adc as f64) / 131072.) - ((self.params.temp.t1 as f64) / 8192.)))
            * ((self.params.temp.t3 as f64) * 16.);
        let t_fine = var1 + var2;

        (t_fine, t_fine / 5120.)
    }

    fn calc_humidity(&self, hum_adc: u32, temperature: f64) -> f64 {
        let var1 = (hum_adc as f64)
            - (((self.params.humidity.h1 as f64) * 16.)
                + (((self.params.humidity.h3 as f64) / 2.) * temperature));
        let var2 = var1
            * (((self.params.humidity.h2 as f64) / 262144.)
                * (1.
                    + (((self.params.humidity.h4 as f64) / 16384.) * temperature)
                    + (((self.params.humidity.h5 as f64) / 1048576.) * temperature * temperature)));
        let var3 = (self.params.humidity.h6 as f64) / 16384.;
        let var4 = (self.params.humidity.h7 as f64) / 2097152.;

        var2 + ((var3 + (var4 * temperature)) * var2 * var2)
    }

    fn calc_pressure(&self, press_adc: u32, t_fine: f64) -> f64 {
        let mut var1 = (t_fine / 2.) - 64000.;
        let mut var2 = var1 * var1 * ((self.params.pressure.p6 as f64) / 131072.);
        var2 = var2 + (var1 * (self.params.pressure.p5 as f64) * 2.);
        var2 = (var2 / 4.) + ((self.params.pressure.p4 as f64) * 65536.);
        var1 = ((((self.params.pressure.p3 as f64) * var1 * var1) / 16384.)
            + ((self.params.pressure.p2 as f64) * var1))
            / 524288.;
        var1 = (1. + (var1 / 32768.)) * (self.params.pressure.p1 as f64);
        let mut press_comp = 1048576. - (press_adc as f64);
        press_comp = ((press_comp - (var2 / 4096.)) * 6250.) / var1;
        var1 = ((self.params.pressure.p9 as f64) * press_comp * press_comp) / 2147483648.;
        var2 = press_comp * ((self.params.pressure.p8 as f64) / 32768.);
        let var3 = (press_comp / 256.)
            * (press_comp / 256.)
            * (press_comp / 256.)
            * ((self.params.pressure.p10 as f64) / 131072.);

        press_comp + (var1 + var2 + var3 + ((self.params.pressure.p7 as f64) * 128.)) / 16.
    }

    fn compute_resistance(&self, gas_adc: u32, gas_range: usize) -> f64 {
        const LOOKUP_K1_RANGE: &[f64; 16] = &[
            0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -0.8, 0.0, 0.0, -0.2, -0.5, 0.0, -1.0, 0.0, 0.0,
        ];
        const LOOKUP_K2_RANGE: &[f64; 16] = &[
            0.0, 0.0, 0.0, 0.0, 0.1, 0.7, 0.0, -0.8, -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let var1 = 1340. + 5. * (self.params.gas.range_switching_error as f64);
        let var2 = var1 * (1. + LOOKUP_K1_RANGE[gas_range] / 100.);
        let var3 = 1. + (LOOKUP_K2_RANGE[gas_range] / 100.);

        1. / (var3
            * 0.000000125
            * ((1 << gas_range) as f64)
            * ((((gas_adc as f64) - 512.) / var2) + 1.))
    }
}
