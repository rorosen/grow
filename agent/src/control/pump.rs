use rppal::gpio::{Gpio, OutputPin};

use super::Error;

pub struct PumpController {
    left_pin: OutputPin,
    right_pin: OutputPin,
}

impl PumpController {
    pub async fn new(left_pin: u8, right_pin: u8) -> Result<Self, Error> {
        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;

        let left_pin = gpio
            .get(left_pin)
            .map_err(Error::GetPinFailed)?
            .into_output_low();

        let right_pin = gpio
            .get(right_pin)
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
}
