use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::config::air::{ExhaustControlConfig, ExhaustControlMode};

use super::control_cyclic;

pub enum ExhaustController {
    Disabled,
    Cyclic {
        pin: OutputPin,
        on_duration: Duration,
        off_duration: Duration,
    },
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

                Ok(Self::Cyclic {
                    pin,
                    on_duration: Duration::from_secs(config.on_duration_secs),
                    off_duration: Duration::from_secs(config.off_duration_secs),
                })
            }
            ExhaustControlMode::Threshold => todo!(),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            ExhaustController::Disabled => (),
            ExhaustController::Cyclic {
                mut pin,
                on_duration,
                off_duration,
            } => control_cyclic(&mut pin, on_duration, off_duration, cancel_token, "exhaust").await,
        }
    }
}
