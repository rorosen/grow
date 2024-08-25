use super::WaterLevelSensor;
use crate::{i2c::I2C, water_level::WaterLevelMeasurement, Error};
use async_trait::async_trait;
use std::{
    path::Path,
    time::{Duration, SystemTime},
};
use tokio_util::sync::CancellationToken;

const IDENTIFICATION_MODEL_ID: u8 = 0xEE;
const RANGE_SEQUENCE_STEP_DSS: u8 = 0x28;
const RANGE_SEQUENCE_STEP_PRE_RANGE: u8 = 0x40;
const RANGE_SEQUENCE_STEP_FINAL_RANGE: u8 = 0x80;

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
pub struct Vl53L0X {
    i2c: I2C,
    stop_variable: Option<u8>,
}

impl Vl53L0X {
    pub async fn new(i2c_path: impl AsRef<Path>, address: u8) -> Result<Self, Error> {
        let mut i2c = I2C::new(i2c_path, address).await?;

        let stop_variable = match Self::init(&mut i2c).await {
            Ok(var) => Some(var),
            Err(err) => {
                log::warn!("Failed to initialize water level sensor: {err:#}");
                None
            }
        };

        Ok(Self { i2c, stop_variable })
    }

    async fn init(i2c: &mut I2C) -> Result<u8, Error> {
        let device_id = i2c.read_reg_byte(REG_IDENTIFICATION_MODEL_ID).await?;
        if device_id != IDENTIFICATION_MODEL_ID {
            return Err(Error::IdentifyFailed);
        }

        let stop_variable = Vl53L0X::init_data(i2c).await?;
        Vl53L0X::init_static(i2c).await?;
        Vl53L0X::perform_ref_calibration(i2c).await?;

        Ok(stop_variable)
    }

    async fn stop_measurement(&mut self, stop_variable: u8) -> Result<(), Error> {
        self.i2c.write_reg_byte(0x80, 0x01).await?;
        self.i2c.write_reg_byte(0xFF, 0x01).await?;
        self.i2c.write_reg_byte(0x00, 0x00).await?;
        self.i2c.write_reg_byte(0x91, stop_variable).await?;
        self.i2c.write_reg_byte(0x00, 0x01).await?;
        self.i2c.write_reg_byte(0xFF, 0x00).await?;
        self.i2c.write_reg_byte(0x80, 0x00).await?;

        Ok(())
    }

    async fn init_data(i2c: &mut I2C) -> Result<u8, Error> {
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

    async fn init_static(i2c: &mut I2C) -> Result<(), Error> {
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

    async fn perform_ref_calibration(i2c: &mut I2C) -> Result<(), Error> {
        Vl53L0X::perform_single_ref_calibration(i2c, 0x01, 0x01 | 0x40).await?;
        Vl53L0X::perform_single_ref_calibration(i2c, 0x02, 0x01).await?;

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
    ) -> Result<(), Error> {
        i2c.write_reg_byte(REG_SYSTEM_SEQUENCE_CONFIG, sequence_config)
            .await?;
        i2c.write_reg_byte(REG_SYSRANGE_START, sysrange_start)
            .await?;

        let mut interrupt_status = i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;

        while (interrupt_status & 0x07) == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            interrupt_status = i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;
        }

        i2c.write_reg_byte(REG_SYSTEM_INTERRUPT_CLEAR, 0x01).await?;
        i2c.write_reg_byte(REG_SYSRANGE_START, 0x00).await?;

        Ok(())
    }
}

#[async_trait]
impl WaterLevelSensor for Vl53L0X {
    async fn measure(
        &mut self,
        cancel_token: CancellationToken,
    ) -> Result<WaterLevelMeasurement, Error> {
        if self.stop_variable.is_none() {
            self.stop_variable = Self::init(&mut self.i2c).await.ok();
        }

        let stop_variable = self.stop_variable.ok_or(Error::NotInit)?;
        // stop any ongoing measurement
        self.stop_measurement(stop_variable).await?;
        // trigger new range measurement
        self.i2c.write_reg_byte(REG_SYSRANGE_START, 0x01).await?;

        // wait for the measurement to start
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(Error::Cancelled);
                },
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    let sysrange_start = self.i2c.read_reg_byte(REG_SYSRANGE_START).await?;
                    if (sysrange_start & 0x01) != 1 {
                        break;
                    }
                }
            }
        }

        // wait for the measurement to finish
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(Error::Cancelled);
                },
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    let interrupt_status = self.i2c.read_reg_byte(REG_RESULT_INTERRUPT_STATUS).await?;
                    if (interrupt_status & 0x07) != 0 {
                        break;
                    }
                }
            }
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

        let measure_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime should be after unix epoch")
            .as_secs();

        let measurement = WaterLevelMeasurement {
            measure_time,
            distance,
        };

        Ok(measurement)
    }
}
