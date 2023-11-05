use rppal::gpio::Gpio;

use super::Error;

pub struct AirPumpController {}

impl AirPumpController {
    pub async fn start(disable: bool, pin: u8) -> Result<(), Error> {
        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin = gpio.get(pin).map_err(Error::GetPinFailed)?.into_output();

        pin.set_reset_on_drop(false);
        if disable {
            log::info!("air pump is disabled permanently");
            pin.set_low();
        } else {
            log::info!("air pump is enabled permanently");
            pin.set_high();
        }

        Ok(())
    }
}
