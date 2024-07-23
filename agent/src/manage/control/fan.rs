use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::config::fan::FanControlConfig;

use super::control_cyclic;

pub enum FanController {
    Disabled,
    Cyclic {
        pin: OutputPin,
        on_duration: Duration,
        off_duration: Duration,
    },
}

impl FanController {
    pub fn new(config: &FanControlConfig) -> Result<Self> {
        if config.enable {
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
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            FanController::Disabled => (),
            FanController::Cyclic {
                mut pin,
                on_duration,
                off_duration,
            } => {
                control_cyclic(
                    &mut pin,
                    on_duration,
                    off_duration,
                    cancel_token,
                    "circulation fan",
                )
                .await
            }
        }
    }
}
