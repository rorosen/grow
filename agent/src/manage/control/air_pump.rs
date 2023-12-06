use crate::error::AppError;
use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};

#[derive(Debug, Parser)]
pub struct AirPumpControlArgs {
    /// Whether the air pump controller is enabled or disabled
    #[arg(
        value_enum,
        id = "air_pump_control_disabled",
        long = "air-pump-control-disabled",
        env = "GROW_AGENT_AIR_PUMP_CONTROL_DISABLED"
    )]
    disabled: bool,

    /// The gpio pin used to control the air pump
    #[arg(
        id = "air_pump_pin",
        long = "air-pump-pin",
        env = "GROW_AGENT_AIR_PUMP_PIN",
        default_value_t = 24
    )]
    pub pin: u8,
}

pub struct AirPumpController {
    pin: OutputPin,
}

impl AirPumpController {
    pub fn set(args: &AirPumpControlArgs) -> Result<(), AppError> {
        let gpio = Gpio::new().map_err(AppError::InitGpioFailed)?;
        let mut pin = gpio
            .get(args.pin)
            .map_err(AppError::GetGpioFailed)?
            .into_output();

        pin.set_reset_on_drop(false);
        pin.set_high();

        Ok(())
    }
}
