use anyhow::{Context, Result};
use rppal::gpio::Gpio;

use crate::config::air_pump::AirPumpControlConfig;

pub struct AirPumpController;

impl AirPumpController {
    pub fn set_pin(config: &AirPumpControlConfig) -> Result<()> {
        if !config.enable {
            log::info!("air pump controller is disabled");
            return Ok(());
        }

        let gpio = Gpio::new().context("failed to initialize GPIO")?;
        let mut pin = gpio
            .get(config.pin)
            .with_context(|| format!("failed to get gpio pin {}", config.pin))?
            .into_output();

        log::info!("activating air pump permanently");
        pin.set_reset_on_drop(false);
        pin.set_high();

        Ok(())
    }
}
