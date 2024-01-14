use std::time::SystemTime;

use grow_utils::api::grow::WaterLevelMeasurement;

use crate::{error::AppError, i2c::I2C};

const IDENTIFICATION_MODEL_ID: u8 = 0xEE;
const RANGE_SEQUENCE_STEP_DSS: u8 = 0x28;
const RANGE_SEQUENCE_STEP_PRE_RANGE: u8 = 0x40;
const RANGE_SEQUENCE_STEP_FINAL_RANGE: u8 = 0x80;
const SENSOR_NAME: &str = "water level (VL53L0X)";

// register addresses
const REG_IDENTIFICATION_MODEL_ID: u8 = 0xC0;
const REG_VHV_CONFIG_PAD_SCL_SDA_EXTSUP_HV: u8 = 0x89;
const REG_SYSTEM_SEQUENCE_CONFIG: u8 = 0x01;
const REG_SYSTEM_INTERRUPT_CONFIG_GPIO: u8 = 0x0A;
const REG_GPIO_HV_MUX_ACTIVE_HIGH: u8 = 0x84;
const REG_SYSTEM_INTERRUPT_CLEAR: u8 = 0x0B;
const REG_RESULT_INTERRUPT_STATUS: u8 = 0x13;
const REG_SYSRANGE_START: u8 = 0x00;
const REG_RESULT_RANGE_STATUS: u8 = 0x14;
const REG_MSRC_CONFIG_CONTROL: u8 = 0x60;
const REG_FINAL_RANGE_CONFIG_MIN_COUNT_RATE_RTN_LIMIT: u8 = 0x44;

// VL53L0X
pub struct WaterLevelSensor {
    i2c: I2C,
    stop_variable: u8,
}

impl WaterLevelSensor {
    pub async fn new(address: u8) -> Result<Self, AppError> {
        let mut i2c = I2C::new(address).await?;

        let device_id = i2c.read_reg_byte(REG_IDENTIFICATION_MODEL_ID).await?;

        if device_id != IDENTIFICATION_MODEL_ID {
            return Err(AppError::IdentifyFailed(SENSOR_NAME.into()));
        }

        log::debug!("identified {} sensor at 0x{:02x}", SENSOR_NAME, address);
        let stop_variable = WaterLevelSensor::init_data(&mut i2c).await?;

        WaterLevelSensor::init_static(&mut i2c).await?;
        WaterLevelSensor::perform_ref_calibration(&mut i2c).await?;
        log::debug!("initialized {} sensor", SENSOR_NAME);

        Ok(Self { i2c, stop_variable })
    }

    pub async fn measure(&mut self) -> Result<WaterLevelMeasurement, AppError> {
        // stop any ongoing measurement
        self.stop_measurement().await?;
        // trigger new range measurement
        self.i2c.write_reg_byte(REG_SYSRANGE_START, 0x01).await?;
        // wait for the measurement to start
        let mut sysrange_start = self.i2c.read_reg_byte(REG_SYSRANGE_START).await?;

        while (sysrange_start & 0x01) == 1 {
            sysrange_start = self.i2c.read_reg_byte(REG_SYSRANGE_START).await?;
        }

        // wait for the measurement to finish
        let mut interrupt_status = self.i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;

        while (interrupt_status & 0x07) == 0 {
            interrupt_status = self.i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;
        }

        // read measurement result
        let distance = self
            .i2c
            .read_reg_u16(REG_RESULT_RANGE_STATUS + 10)
            .await?
            .into();

        // clear interrupt
        self.i2c
            .write_reg_byte(REG_SYSTEM_INTERRUPT_CLEAR, 0x01)
            .await?;

        Ok(WaterLevelMeasurement {
            measure_time: Some(SystemTime::now().into()),
            distance,
        })
    }

    async fn stop_measurement(&mut self) -> Result<(), AppError> {
        self.i2c.write_reg_byte(0x80, 0x01).await?;
        self.i2c.write_reg_byte(0xFF, 0x01).await?;
        self.i2c.write_reg_byte(0x00, 0x00).await?;
        self.i2c.write_reg_byte(0x91, self.stop_variable).await?;
        self.i2c.write_reg_byte(0x00, 0x01).await?;
        self.i2c.write_reg_byte(0xFF, 0x00).await?;
        self.i2c.write_reg_byte(0x80, 0x00).await?;

        Ok(())
    }

    async fn init_data(i2c: &mut I2C) -> Result<u8, AppError> {
        // set 2v8 mode
        i2c.set_reg_bits(REG_VHV_CONFIG_PAD_SCL_SDA_EXTSUP_HV, 0x01)
            .await?;

        // set i2c standard mode
        i2c.write_reg_byte(0x88, 0x00).await?;
        i2c.write_reg_byte(0x80, 0x01).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x00, 0x00).await?;
        let stop_variable = i2c.read_reg_byte(0x91).await?;
        i2c.write_reg_byte(0x00, 0x01).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x80, 0x00).await?;

        // disable SIGNAL_RATE_MSRC (bit 1) and SIGNAL_RATE_PRE_RANGE (bit 4) limit checks
        i2c.set_reg_bits(REG_MSRC_CONFIG_CONTROL, 0x12).await?;

        // set final range signal rate limit to 0.25 million counts per second
        // Q9.7 fixed point format (9 integer bits, 7 fractional bits)
        //   writeReg16Bit(FINAL_RANGE_CONFIG_MIN_COUNT_RATE_RTN_LIMIT, limit_Mcps * (1 << 7));
        i2c.write_reg_u16(REG_FINAL_RANGE_CONFIG_MIN_COUNT_RATE_RTN_LIMIT, 208)
            .await?;

        Ok(stop_variable)
    }

    async fn init_static(i2c: &mut I2C) -> Result<(), AppError> {
        // load default tuning settings
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x00, 0x00).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x09, 0x00).await?;
        i2c.write_reg_byte(0x10, 0x00).await?;
        i2c.write_reg_byte(0x11, 0x00).await?;
        i2c.write_reg_byte(0x24, 0x01).await?;
        i2c.write_reg_byte(0x25, 0xFF).await?;
        i2c.write_reg_byte(0x75, 0x00).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x4E, 0x2C).await?;
        i2c.write_reg_byte(0x48, 0x00).await?;
        i2c.write_reg_byte(0x30, 0x20).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x30, 0x09).await?;
        i2c.write_reg_byte(0x54, 0x00).await?;
        i2c.write_reg_byte(0x31, 0x04).await?;
        i2c.write_reg_byte(0x32, 0x03).await?;
        i2c.write_reg_byte(0x40, 0x83).await?;
        i2c.write_reg_byte(0x46, 0x25).await?;
        i2c.write_reg_byte(0x60, 0x00).await?;
        i2c.write_reg_byte(0x27, 0x00).await?;
        i2c.write_reg_byte(0x50, 0x06).await?;
        i2c.write_reg_byte(0x51, 0x00).await?;
        i2c.write_reg_byte(0x52, 0x96).await?;
        i2c.write_reg_byte(0x56, 0x08).await?;
        i2c.write_reg_byte(0x57, 0x30).await?;
        i2c.write_reg_byte(0x61, 0x00).await?;
        i2c.write_reg_byte(0x62, 0x00).await?;
        i2c.write_reg_byte(0x64, 0x00).await?;
        i2c.write_reg_byte(0x65, 0x00).await?;
        i2c.write_reg_byte(0x66, 0xA0).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x22, 0x32).await?;
        i2c.write_reg_byte(0x47, 0x14).await?;
        i2c.write_reg_byte(0x49, 0xFF).await?;
        i2c.write_reg_byte(0x4A, 0x00).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x7A, 0x0A).await?;
        i2c.write_reg_byte(0x7B, 0x00).await?;
        i2c.write_reg_byte(0x78, 0x21).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x23, 0x34).await?;
        i2c.write_reg_byte(0x42, 0x00).await?;
        i2c.write_reg_byte(0x44, 0xFF).await?;
        i2c.write_reg_byte(0x45, 0x26).await?;
        i2c.write_reg_byte(0x46, 0x05).await?;
        i2c.write_reg_byte(0x40, 0x40).await?;
        i2c.write_reg_byte(0x0E, 0x06).await?;
        i2c.write_reg_byte(0x20, 0x1A).await?;
        i2c.write_reg_byte(0x43, 0x40).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x34, 0x03).await?;
        i2c.write_reg_byte(0x35, 0x44).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x31, 0x04).await?;
        i2c.write_reg_byte(0x4B, 0x09).await?;
        i2c.write_reg_byte(0x4C, 0x05).await?;
        i2c.write_reg_byte(0x4D, 0x04).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x44, 0x00).await?;
        i2c.write_reg_byte(0x45, 0x20).await?;
        i2c.write_reg_byte(0x47, 0x08).await?;
        i2c.write_reg_byte(0x48, 0x28).await?;
        i2c.write_reg_byte(0x67, 0x00).await?;
        i2c.write_reg_byte(0x70, 0x04).await?;
        i2c.write_reg_byte(0x71, 0x01).await?;
        i2c.write_reg_byte(0x72, 0xFE).await?;
        i2c.write_reg_byte(0x76, 0x00).await?;
        i2c.write_reg_byte(0x77, 0x00).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x0D, 0x01).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x80, 0x01).await?;
        i2c.write_reg_byte(0x01, 0xF8).await?;
        i2c.write_reg_byte(0xFF, 0x01).await?;
        i2c.write_reg_byte(0x8E, 0x01).await?;
        i2c.write_reg_byte(0x00, 0x01).await?;
        i2c.write_reg_byte(0xFF, 0x00).await?;
        i2c.write_reg_byte(0x80, 0x00).await?;

        // configure interrupt
        i2c.write_reg_byte(REG_SYSTEM_INTERRUPT_CONFIG_GPIO, 0x04)
            .await?;

        let gpio_hv_mux_active_high = i2c.read_reg_byte(REG_GPIO_HV_MUX_ACTIVE_HIGH).await?;

        i2c.write_reg_byte(REG_GPIO_HV_MUX_ACTIVE_HIGH, gpio_hv_mux_active_high & !0x10)
            .await?;

        i2c.write_reg_byte(REG_SYSTEM_INTERRUPT_CLEAR, 0x01).await?;

        // enable steps in the sequence
        i2c.write_reg_byte(
            REG_SYSTEM_SEQUENCE_CONFIG,
            RANGE_SEQUENCE_STEP_DSS
                + RANGE_SEQUENCE_STEP_PRE_RANGE
                + RANGE_SEQUENCE_STEP_FINAL_RANGE,
        )
        .await?;

        Ok(())
    }

    async fn perform_ref_calibration(i2c: &mut I2C) -> Result<(), AppError> {
        WaterLevelSensor::perform_single_ref_calibration(i2c, 0x01, 0x01 | 0x40).await?;
        WaterLevelSensor::perform_single_ref_calibration(i2c, 0x02, 0x01 | 0x00).await?;

        // restore sequence steps
        i2c.write_reg_byte(
            REG_SYSTEM_SEQUENCE_CONFIG,
            RANGE_SEQUENCE_STEP_DSS
                + RANGE_SEQUENCE_STEP_PRE_RANGE
                + RANGE_SEQUENCE_STEP_FINAL_RANGE,
        )
        .await?;

        Ok(())
    }

    async fn perform_single_ref_calibration(
        i2c: &mut I2C,
        sequence_config: u8,
        sysrange_start: u8,
    ) -> Result<(), AppError> {
        i2c.write_reg_byte(REG_SYSTEM_SEQUENCE_CONFIG, sequence_config)
            .await?;
        i2c.write_reg_byte(REG_SYSRANGE_START, sysrange_start)
            .await?;

        let mut interrupt_status = i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;

        while (interrupt_status & 0x07) == 0 {
            interrupt_status = i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;
        }

        i2c.write_reg_byte(REG_SYSTEM_INTERRUPT_CLEAR, 0x01).await?;
        i2c.write_reg_byte(REG_SYSRANGE_START, 0x00).await?;

        Ok(())
    }
}
