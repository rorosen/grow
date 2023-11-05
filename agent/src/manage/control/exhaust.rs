use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};

use super::Error;

#[derive(Debug, Parser)]
pub struct ExhaustControlArgs {
    /// The gpio pin used to enable the exhaust fan in slow mode
    #[arg(
        id = "exhaust_control_pin_slow",
        long = "exhaust-control-pin-slow",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_SLOW",
        default_value_t = 25
    )]
    pub pin_slow: u8,

    /// The gpio pin used to enable the exhaust fan in fast mode (not implemented)
    #[arg(
        id = "exhaust_control_pin_fast",
        long = "exhaust-control-pin-fast",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_FAST",
        default_value_t = 26
    )]
    pub pin_fast: u8,
}

pub struct ExhaustController {
    pin: OutputPin,
}

impl ExhaustController {
    pub async fn new(pin: u8) -> Result<Self, Error> {
        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let pin = gpio
            .get(pin)
            .map_err(Error::GetPinFailed)?
            .into_output_low();

        Ok(Self { pin })
    }

    pub fn activate(&mut self) {
        self.pin.set_high();
    }

    pub fn activate_permanent(&mut self) {
        self.pin.set_reset_on_drop(false);
        self.activate();
    }

    pub fn deactivate(&mut self) {
        self.pin.set_low();
    }

    pub fn deactivate_permanent(&mut self) {
        self.pin.set_reset_on_drop(false);
        self.deactivate();
    }
}
