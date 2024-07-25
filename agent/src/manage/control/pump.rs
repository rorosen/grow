use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::PumpControlConfig;

pub enum PumpController {
    Enabled { pin: OutputPin },
    Disabled,
}

impl PumpController {
    pub fn new(config: &PumpControlConfig) -> Result<Self> {
        if config.enable {
            let gpio = Gpio::new().context("failed to initialize GPIO")?;
            let pin = gpio
                .get(config.pin)
                .with_context(|| format!("failed to get gpio pin {} (left)", config.pin))?
                .into_output();

            Ok(Self::Enabled { pin })
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, _: CancellationToken) {
        match self {
            PumpController::Enabled { .. } => todo!("implement pump controller run"),
            PumpController::Disabled => (),
        }
    }
}
