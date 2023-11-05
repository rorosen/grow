use rppal::gpio::{Gpio, OutputPin};

use super::Error;

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
