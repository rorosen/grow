use anyhow::{Context, Result};
use rppal::gpio::Gpio;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::config::air::{ExhaustControlConfig, ExhaustControlMode};

use super::CyclicController;

pub enum ExhaustController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl ExhaustController {
    pub fn new(config: &ExhaustControlConfig) -> Result<Self> {
        match config.mode {
            ExhaustControlMode::Off => Ok(Self::Disabled),
            ExhaustControlMode::Cyclic => {
                let gpio = Gpio::new().context("failed to initialize GPIO")?;
                let pin = gpio
                    .get(config.pin)
                    .with_context(|| format!("failed to get gpio pin {}", config.pin))?
                    .into_output();
                let controller = CyclicController::new(
                    pin,
                    Duration::from_secs(config.on_duration_secs),
                    Duration::from_secs(config.off_duration_secs),
                );

                Ok(Self::Cyclic { controller })
            }
            ExhaustControlMode::Threshold => todo!(),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            ExhaustController::Disabled => (),
            ExhaustController::Cyclic { mut controller } => {
                controller.run(cancel_token, "exhaust fan").await
            }
        }
    }
}
