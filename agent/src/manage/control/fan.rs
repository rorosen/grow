use anyhow::{Context, Result};
use rppal::gpio::Gpio;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::config::fan::FanControlConfig;

use super::CyclicController;

pub enum FanController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl FanController {
    pub fn new(config: &FanControlConfig) -> Result<Self> {
        if config.enable {
            let gpio = Gpio::new().context("Failed to initialize GPIO")?;
            let pin = gpio
                .get(config.pin)
                .with_context(|| format!("Failed to get gpio pin {}", config.pin))?
                .into_output();
            let controller = CyclicController::new(
                pin,
                Duration::from_secs(config.on_duration_secs),
                Duration::from_secs(config.off_duration_secs),
            );

            Ok(Self::Cyclic { controller })
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            FanController::Disabled => (),
            FanController::Cyclic { mut controller } => controller.run(cancel_token, "fan").await,
        }
    }
}
