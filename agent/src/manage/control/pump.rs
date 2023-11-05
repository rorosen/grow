use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};

use super::Error;

#[derive(Debug, Parser)]
pub struct PumpControlArgs {
    /// The gpio pin used to disable the left pump
    #[arg(
        id = "pump.control-pin-left",
        long = "pump-control-pin-left",
        env = "GROW_AGENT_PUMP_CONTROL_PIN_LEFT",
        default_value_t = 17
    )]
    pub pin_left: u8,

    /// The gpio pin used to disable the right pump
    #[arg(
        id = "pump.control-pin-right",
        long = "pump-control-pin-right",
        env = "GROW_AGENT_PUMP_CONTROL_PIN_RIGHT",
        default_value_t = 22
    )]
    pub pin_right: u8,
}

pub struct PumpController {
    left_pin: OutputPin,
    right_pin: OutputPin,
}

impl PumpController {
    pub fn new(args: PumpControlArgs) -> Result<Self, Error> {
        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;

        let left_pin = gpio
            .get(args.pin_left)
            .map_err(Error::GetPinFailed)?
            .into_output_low();

        let right_pin = gpio
            .get(args.pin_right)
            .map_err(Error::GetPinFailed)?
            .into_output_low();

        Ok(Self {
            left_pin,
            right_pin,
        })
    }

    pub fn activate_left(&mut self) {
        self.left_pin.set_high();
    }

    pub fn activate_right(&mut self) {
        self.right_pin.set_high();
    }

    pub fn deactivate_left(&mut self) {
        self.left_pin.set_low();
    }

    pub fn deactivate_right(&mut self) {
        self.right_pin.set_low();
    }
}
