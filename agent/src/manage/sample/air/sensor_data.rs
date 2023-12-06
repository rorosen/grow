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

pub struct SensorData {
    pub temp_adc: u32,
    pub press_adc: u32,
    pub hum_adc: u32,
    pub gas_adc: u32,
    pub gas_range: u8,
    pub gas_valid: bool,
    pub heater_stable: bool,
}

impl SensorData {
    pub fn new(data: &[u8; 17]) -> Self {
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
            gas_valid: is_gas_valid(data[OFFSET_GAS_RLSB]),
            heater_stable: is_heater_stable(data[OFFSET_GAS_RLSB]),
        }
    }
}

fn is_gas_valid(data: u8) -> bool {
    (data & MASK_GAS_VALID) > 0
}

fn is_heater_stable(data: u8) -> bool {
    (data & MASK_HEATER_STABLE) > 0
}
