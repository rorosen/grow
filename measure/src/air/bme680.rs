use crate::{air::AirSensor, i2c::I2C, Error};
use async_trait::async_trait;
use chrono::Utc;
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use super::AirMeasurement;

const TEMPERATURE_MAX: u16 = 400;
const GAS_WAIT_MS_MAX: u16 = 4032;
const GAS_WAIT_VALUE_MAX: u8 = 0xFF;

const OFFSET_T1_LSB: usize = 31;
const OFFSET_T1_MSB: usize = 32;
const OFFSET_T2_LSB: usize = 0;
const OFFSET_T2_MSB: usize = 1;
const OFFSET_T3: usize = 2;

const OFFSET_P1_LSB: usize = 4;
const OFFSET_P1_MSB: usize = 5;
const OFFSET_P2_LSB: usize = 6;
const OFFSET_P2_MSB: usize = 7;
const OFFSET_P3: usize = 8;
const OFFSET_P4_LSB: usize = 10;
const OFFSET_P4_MSB: usize = 11;
const OFFSET_P5_LSB: usize = 12;
const OFFSET_P5_MSB: usize = 13;
const OFFSET_P6: usize = 15;
const OFFSET_P7: usize = 14;
const OFFSET_P8_LSB: usize = 18;
const OFFSET_P8_MSB: usize = 19;
const OFFSET_P9_LSB: usize = 20;
const OFFSET_P9_MSB: usize = 21;
const OFFSET_P10: usize = 22;

const OFFSET_H1_LSB: usize = 24;
const OFFSET_H1_MSB: usize = 25;
const OFFSET_H2_LSB: usize = 24;
const OFFSET_H2_MSB: usize = 23;
const OFFSET_H3: usize = 26;
const OFFSET_H4: usize = 27;
const OFFSET_H5: usize = 28;
const OFFSET_H6: usize = 29;
const OFFSET_H7: usize = 30;

const OFFSET_GH1: usize = 35;
const OFFSET_GH2_LSB: usize = 33;
const OFFSET_GH2_MSB: usize = 34;
const OFFSET_GH3: usize = 36;
const OFFSET_RES_HEAT_VAL: usize = 37;
const OFFSET_RES_HEAT_RANGE: usize = 39;
const OFFSET_RANGE_SWITCHING_ERROR: usize = 41;

const MASK_H1_LSB: u16 = 0x0F;
const MASK_HEAT_RANGE: u8 = 0x30;
const MASK_RANGE_SWITCHING_ERROR: u8 = 0xF0;

const OFFSET_PRESS_MSB: usize = 2;
const OFFSET_PRESS_LSB: usize = 3;
const OFFSET_PRESS_XLSB: usize = 4;
const OFFSET_TEMP_MSB: usize = 5;
const OFFSET_TEMP_LSB: usize = 6;
const OFFSET_TEMP_XLSB: usize = 7;
const OFFSET_HUM_MSB: usize = 8;
const OFFSET_HUM_LSB: usize = 9;
const OFFSET_GAS_RMSB: usize = 13;
const OFFSET_GAS_RLSB: usize = 14;
const MASK_GAS_RANGE: u8 = 0x0F;
const MASK_GAS_VALID: u8 = 0x20;
const MASK_HEATER_STABLE: u8 = 0x10;

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

struct SensorData {
    temp_adc: u32,
    press_adc: u32,
    hum_adc: u32,
    gas_adc: u32,
    gas_range: u8,
    #[allow(dead_code)]
    gas_valid: bool,
    #[allow(dead_code)]
    heater_stable: bool,
}

impl SensorData {
    fn new(data: &[u8; 17]) -> Self {
        Self {
            temp_adc: ((data[OFFSET_TEMP_MSB] as u32) << 12)
                | ((data[OFFSET_TEMP_LSB] as u32) << 4)
                | ((data[OFFSET_TEMP_XLSB] as u32) >> 4),
            press_adc: ((data[OFFSET_PRESS_MSB] as u32) << 12)
                | ((data[OFFSET_PRESS_LSB] as u32) << 4)
                | ((data[OFFSET_PRESS_XLSB] as u32) >> 4),
            hum_adc: ((data[OFFSET_HUM_MSB] as u32) << 8) | (data[OFFSET_HUM_LSB] as u32),
            gas_adc: ((data[OFFSET_GAS_RMSB] as u32) << 2) | ((data[OFFSET_GAS_RLSB] as u32) >> 6),
            gas_range: data[OFFSET_GAS_RLSB] & MASK_GAS_RANGE,
            gas_valid: (data[OFFSET_GAS_RLSB] & MASK_GAS_VALID) > 0,
            heater_stable: (data[OFFSET_GAS_RLSB] & MASK_HEATER_STABLE) > 0,
        }
    }
}

struct TempParams {
    t1: u16,
    t2: i16,
    t3: i8,
}

struct PressureParams {
    p1: u16,
    p2: i16,
    p3: i8,
    p4: i16,
    p5: i16,
    p6: i8,
    p7: i8,
    p8: i16,
    p9: i16,
    p10: u8,
}

struct HumidityParams {
    h1: u16,
    h2: u16,
    h3: i8,
    h4: i8,
    h5: i8,
    h6: u8,
    h7: i8,
}

struct GasParams {
    gh1: i8,
    gh2: i16,
    gh3: i8,
    heat_range: u8,
    heat_val: i8,
    range_switching_error: i8,
}

struct Params {
    temp: TempParams,
    pressure: PressureParams,
    humidity: HumidityParams,
    gas: GasParams,
}

impl Params {
    fn new(values: &[u8; 42]) -> Self {
        Self {
            temp: TempParams {
                t1: ((values[OFFSET_T1_MSB] as u16) << 8) | (values[OFFSET_T1_LSB] as u16),
                t2: (((values[OFFSET_T2_MSB] as u16) << 8) | (values[OFFSET_T2_LSB] as u16)) as i16,
                t3: values[OFFSET_T3] as i8,
            },
            pressure: PressureParams {
                p1: ((values[OFFSET_P1_MSB] as u16) << 8) | (values[OFFSET_P1_LSB] as u16),
                p2: (((values[OFFSET_P2_MSB] as u16) << 8) | (values[OFFSET_P2_LSB] as u16)) as i16,
                p3: values[OFFSET_P3] as i8,
                p4: (((values[OFFSET_P4_MSB] as u16) << 8) | (values[OFFSET_P4_LSB] as u16)) as i16,
                p5: (((values[OFFSET_P5_MSB] as u16) << 8) | (values[OFFSET_P5_LSB] as u16)) as i16,
                p6: values[OFFSET_P6] as i8,
                p7: values[OFFSET_P7] as i8,
                p8: (((values[OFFSET_P8_MSB] as u16) << 8) | (values[OFFSET_P8_LSB] as u16)) as i16,
                p9: (((values[OFFSET_P9_MSB] as u16) << 8) | (values[OFFSET_P9_LSB] as u16)) as i16,
                p10: values[OFFSET_P10],
            },
            humidity: HumidityParams {
                h1: ((values[OFFSET_H1_MSB] as u16) << 4)
                    | ((values[OFFSET_H1_LSB] as u16) & MASK_H1_LSB),
                h2: ((values[OFFSET_H2_MSB] as u16) << 4) | ((values[OFFSET_H2_LSB] as u16) >> 4),
                h3: values[OFFSET_H3] as i8,
                h4: values[OFFSET_H4] as i8,
                h5: values[OFFSET_H5] as i8,
                h6: values[OFFSET_H6],
                h7: values[OFFSET_H7] as i8,
            },
            gas: GasParams {
                gh1: values[OFFSET_GH1] as i8,
                gh2: (((values[OFFSET_GH2_MSB] as u16) << 8) | (values[OFFSET_GH2_LSB]) as u16)
                    as i16,
                gh3: values[OFFSET_GH3] as i8,
                heat_range: (values[OFFSET_RES_HEAT_RANGE] & MASK_HEAT_RANGE) >> 4,
                heat_val: values[OFFSET_RES_HEAT_VAL] as i8,
                range_switching_error: ((values[OFFSET_RANGE_SWITCHING_ERROR]
                    & MASK_RANGE_SWITCHING_ERROR)
                    >> 4) as i8,
            },
        }
    }

    fn calc_heat_resistance(&self, ambient_temperature: i8, mut temperature: u16) -> u8 {
        if temperature > TEMPERATURE_MAX {
            temperature = TEMPERATURE_MAX;
        }

        let var1 = ((self.gas.gh1 as f64) / 16.) + 49.;
        let var2 = (((self.gas.gh2 as f64) / 32768.) * 0.00005) + 0.00235;
        let var3 = (self.gas.gh3 as f64) / 1024.;
        let var4 = var1 * (1. + (var2 * (temperature as f64)));
        let var5 = var4 + (var3 * (ambient_temperature as f64));
        (3.4 * (var5
            * (4.
                / (4. + (self.gas.heat_range as f64) * (1. + (self.gas.heat_val as f64) * 0.002)))
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
        let var1 = (((temp_adc as f64) / 16384.) - ((self.temp.t1 as f64) / 1024.))
            * (self.temp.t2 as f64);
        let var2 = ((((temp_adc as f64) / 131072.) - ((self.temp.t1 as f64) / 8192.))
            * (((temp_adc as f64) / 131072.) - ((self.temp.t1 as f64) / 8192.)))
            * ((self.temp.t3 as f64) * 16.);
        let t_fine = var1 + var2;

        (t_fine, t_fine / 5120.)
    }

    fn calc_humidity(&self, hum_adc: u32, temperature: f64) -> f64 {
        let var1 = (hum_adc as f64)
            - (((self.humidity.h1 as f64) * 16.)
                + (((self.humidity.h3 as f64) / 2.) * temperature));
        let var2 = var1
            * (((self.humidity.h2 as f64) / 262144.)
                * (1.
                    + (((self.humidity.h4 as f64) / 16384.) * temperature)
                    + (((self.humidity.h5 as f64) / 1048576.) * temperature * temperature)));
        let var3 = (self.humidity.h6 as f64) / 16384.;
        let var4 = (self.humidity.h7 as f64) / 2097152.;

        var2 + ((var3 + (var4 * temperature)) * var2 * var2)
    }

    fn calc_pressure(&self, press_adc: u32, t_fine: f64) -> f64 {
        let mut var1 = (t_fine / 2.) - 64000.;
        let mut var2 = var1 * var1 * ((self.pressure.p6 as f64) / 131072.);
        var2 += var1 * (self.pressure.p5 as f64) * 2.;
        var2 = (var2 / 4.) + ((self.pressure.p4 as f64) * 65536.);
        var1 = ((((self.pressure.p3 as f64) * var1 * var1) / 16384.)
            + ((self.pressure.p2 as f64) * var1))
            / 524288.;
        var1 = (1. + (var1 / 32768.)) * (self.pressure.p1 as f64);
        let mut press_comp = 1048576. - (press_adc as f64);
        press_comp = ((press_comp - (var2 / 4096.)) * 6250.) / var1;
        var1 = ((self.pressure.p9 as f64) * press_comp * press_comp) / 2147483648.;
        var2 = press_comp * ((self.pressure.p8 as f64) / 32768.);
        let var3 = (press_comp / 256.)
            * (press_comp / 256.)
            * (press_comp / 256.)
            * ((self.pressure.p10 as f64) / 131072.);

        press_comp + (var1 + var2 + var3 + ((self.pressure.p7 as f64) * 128.)) / 16.
    }

    fn compute_resistance(&self, gas_adc: u32, gas_range: usize) -> f64 {
        const LOOKUP_K1_RANGE: &[f64; 16] = &[
            0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -0.8, 0.0, 0.0, -0.2, -0.5, 0.0, -1.0, 0.0, 0.0,
        ];
        const LOOKUP_K2_RANGE: &[f64; 16] = &[
            0.0, 0.0, 0.0, 0.0, 0.1, 0.7, 0.0, -0.8, -0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];

        let var1 = 1340. + 5. * (self.gas.range_switching_error as f64);
        let var2 = var1 * (1. + LOOKUP_K1_RANGE[gas_range] / 100.);
        let var3 = 1. + (LOOKUP_K2_RANGE[gas_range] / 100.);

        1. / (var3
            * 0.000000125
            * ((1 << gas_range) as f64)
            * ((((gas_adc as f64) - 512.) / var2) + 1.))
    }
}

/// BME680
pub struct Bme680 {
    i2c: I2C,
    params: Option<Params>,
}

impl Bme680 {
    pub async fn new(i2c_path: impl AsRef<Path>, address: u8) -> Result<Self, Error> {
        let mut i2c = I2C::new(i2c_path, address).await?;
        let params = match Self::init_params(&mut i2c).await {
            Ok(params) => Some(params),
            Err(err) => {
                log::warn!("Failed to initialize BME680 at address 0x{address:02x}: {err}");
                None
            }
        };

        Ok(Self { i2c, params })
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
            return Err(Error::NotInit);
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
            return Err(Error::Identify);
        }

        i2c.write_reg_byte(REG_RESET, CMD_SOFT_RESET).await?;

        tokio::time::sleep(Duration::from_millis(5)).await;

        let mut params = [0; PARAMS_SIZE1 + PARAMS_SIZE2 + PARAMS_SIZE3];
        let (params1, rest) = params.split_at_mut(PARAMS_SIZE1);
        let (params2, params3) = rest.split_at_mut(PARAMS_SIZE2);

        i2c.read_reg_bytes(REG_PARAMS1, params1).await?;
        i2c.read_reg_bytes(REG_PARAMS2, params2).await?;
        i2c.read_reg_bytes(REG_PARAMS3, params3).await?;

        Ok(Params::new(&params))
    }
}

#[async_trait]
impl AirSensor for Bme680 {
    async fn measure(
        &mut self,
        label: String,
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
        let measure_time = Utc::now().timestamp();
        let params = self.params.as_ref().ok_or(Error::NotInit)?;
        let (t_fine, temperature) = params.calc_temperature(data.temp_adc);
        let humidity = params.calc_humidity(data.hum_adc, temperature);
        let pressure = params.calc_pressure(data.press_adc, t_fine) / 100.;
        let resistance = params.compute_resistance(data.gas_adc, data.gas_range as usize);
        let measurement = AirMeasurement::new(measure_time, label)
            .temperature(temperature)
            .humidity(humidity)
            .pressure(pressure)
            .resistance(resistance);

        Ok(measurement)
    }
}
