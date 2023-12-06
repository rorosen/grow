#![allow(dead_code)]
use rppal::gpio::{Gpio, OutputPin};

use crate::error::AppError;

pub struct PumpController {
    left_pin: OutputPin,
    right_pin: OutputPin,
}

impl PumpController {
    pub fn new(pin_left: u8, pin_right: u8) -> Result<Self, AppError> {
        let gpio = Gpio::new().map_err(AppError::InitGpioFailed)?;

        let left_pin = gpio
            .get(pin_left)
            .map_err(AppError::GetGpioFailed)?
            .into_output_low();

        let right_pin = gpio
            .get(pin_right)
            .map_err(AppError::GetGpioFailed)?
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
