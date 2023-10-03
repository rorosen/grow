use crate::periph::i2c::I2C;
use clap::Parser;

use super::Error;

const ADDRESS_REFERENCE_REG: u8 = 0xC0;

#[derive(Debug, Parser)]
pub struct WaterLevelSampleArgs {
    /// Whether to disable the water level sampler
    #[arg(
        id = "water_sample_disable",
        long = "water-sample-disable",
        env = "GROW_AGENT_WATER_SAMPLE_DISABLE"
    )]
    pub disable: bool,

    /// The I2C address of the left water level sensor
    #[arg(
        id = "water_sample_address_left",
        long = "water-sample-address-left",
        env = "GROW_AGENT_WATER_SAMPLE_ADDRESS_LEFT",
        default_value_t = 0x29
    )]
    pub address_left: u8,

    /// The I2C address of the right water level sensor
    #[arg(
        id = "water_sample_address_right",
        long = "water-sample-address-right",
        env = "GROW_AGENT_WATER_SAMPLE_ADDRESS_RIGHT",
        default_value_t = 0x29
    )]
    pub address_right: u8,
}

pub struct WaterLevelSampler {
    i2c_left: I2C,
    // i2c_right: I2C,
}

impl WaterLevelSampler {
    pub async fn start(args: WaterLevelSampleArgs) -> Result<(), Error> {
        let mut i2c_left = I2C::new(args.address_left)
            .await
            .map_err(Error::InitI2cFailed)?;
        // let i2c_right = I2C::new(args.address_right)
        //     .await
        //     .map_err(Error::InitI2CFailed)?;

        let data = i2c_left
            .read_reg(0xC0)
            .await
            .map_err(Error::I2cActionFailed)?;

        println!("0x{data:02x}");

        Ok(())
    }
}
