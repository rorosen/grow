use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::PumpControlConfig;

pub enum PumpController {
    Enabled {
        left_pin: OutputPin,
        right_pin: OutputPin,
    },
    Disabled,
}

impl PumpController {
    pub fn new(config: &PumpControlConfig) -> Result<Self> {
        if config.enable {
            let gpio = Gpio::new().context("failed to initialize GPIO")?;
            let left_pin = gpio
                .get(config.left_pin)
                .with_context(|| format!("failed to get gpio pin {} (left)", config.left_pin))?
                .into_output();
            let right_pin = gpio
                .get(config.right_pin)
                .with_context(|| format!("failed to get gpio pin {} (right)", config.right_pin))?
                .into_output();

            Ok(Self::Enabled {
                left_pin,
                right_pin,
            })
        } else {
            Ok(Self::Disabled)
        }
    }

    // TODO: implement run
    pub async fn run(self, _: CancellationToken) {}
}
