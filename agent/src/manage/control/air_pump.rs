use clap::Parser;
use rppal::gpio::Gpio;

use super::Error;

#[derive(Debug, Parser)]
pub struct AirPumpControlArgs {
    /// Whether to disable the air pump
    #[arg(
        id = "air_pump_disable",
        long = "air-pump-disable",
        env = "GROW_AGENT_AIR_PUMP_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to control the air pump
    #[arg(
        id = "air_pump_pin",
        long = "air-pump-pin",
        env = "GROW_AGENT_AIR_PUMP_PIN",
        default_value_t = 24
    )]
    pub pin: u8,
}

impl AirPumpControlArgs {
    pub fn set_air_pump(&self) -> Result<(), Error> {
        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin = gpio
            .get(self.pin)
            .map_err(Error::GetPinFailed)?
            .into_output();

        pin.set_reset_on_drop(false);
        if self.disable {
            log::info!("air pump is disabled");
            pin.set_low();
        } else {
            log::info!("air pump is enabled");
            pin.set_high();
        }

        Ok(())
    }
}
