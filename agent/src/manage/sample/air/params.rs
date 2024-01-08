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

pub struct TempParams {
    pub t1: u16,
    pub t2: i16,
    pub t3: i8,
}

pub struct PressureParams {
    pub p1: u16,
    pub p2: i16,
    pub p3: i8,
    pub p4: i16,
    pub p5: i16,
    pub p6: i8,
    pub p7: i8,
    pub p8: i16,
    pub p9: i16,
    pub p10: u8,
}

pub struct HumidityParams {
    pub h1: u16,
    pub h2: u16,
    pub h3: i8,
    pub h4: i8,
    pub h5: i8,
    pub h6: u8,
    pub h7: i8,
}

pub struct GasParams {
    pub gh1: i8,
    pub gh2: i16,
    pub gh3: i8,
    pub heat_range: u8,
    pub heat_val: i8,
    pub range_switching_error: i8,
}

pub struct Params {
    pub temp: TempParams,
    pub pressure: PressureParams,
    pub humidity: HumidityParams,
    pub gas: GasParams,
}

impl Params {
    pub fn new(values: &[u8; 42]) -> Self {
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

    pub fn calc_heat_resistance(&self, ambient_temperature: i8, mut temperature: u16) -> u8 {
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
                / (4.
                    + (self.gas.heat_range as f64)
                        * (1. / 1. + (self.gas.heat_val as f64) * 0.002)))
            - 25.)) as u8
    }

    pub fn calc_gas_wait(&self, mut duration: u16) -> u8 {
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

    pub fn calc_temperature(&self, temp_adc: u32) -> (f64, f64) {
        let var1 = (((temp_adc as f64) / 16384.) - ((self.temp.t1 as f64) / 1024.))
            * (self.temp.t2 as f64);
        let var2 = ((((temp_adc as f64) / 131072.) - ((self.temp.t1 as f64) / 8192.))
            * (((temp_adc as f64) / 131072.) - ((self.temp.t1 as f64) / 8192.)))
            * ((self.temp.t3 as f64) * 16.);
        let t_fine = var1 + var2;

        (t_fine, t_fine / 5120.)
    }

    pub fn calc_humidity(&self, hum_adc: u32, temperature: f64) -> f64 {
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

    pub fn calc_pressure(&self, press_adc: u32, t_fine: f64) -> f64 {
        let mut var1 = (t_fine / 2.) - 64000.;
        let mut var2 = var1 * var1 * ((self.pressure.p6 as f64) / 131072.);
        var2 = var2 + (var1 * (self.pressure.p5 as f64) * 2.);
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

    pub fn compute_resistance(&self, gas_adc: u32, gas_range: usize) -> f64 {
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
