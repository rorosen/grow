use anyhow::{Context, Result};
use chrono::NaiveTime;
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use crate::config::light::LightControlConfig;

use super::control_time_based;

pub enum LightController {
    Disabled,
    Time {
        pin: OutputPin,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    },
}

impl LightController {
    pub fn new(config: &LightControlConfig) -> Result<Self> {
        if config.enable {
            let gpio = Gpio::new().context("failed to initialize GPIO")?;
            let pin = gpio
                .get(config.pin)
                .with_context(|| format!("failed to get gpio pin {}", config.pin))?
                .into_output();

            Ok(Self::Time {
                pin,
                activate_time: config.activate_time,
                deactivate_time: config.deactivate_time,
            })
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            LightController::Disabled => Ok(()),
            LightController::Time {
                mut pin,
                activate_time,
                deactivate_time,
            } => {
                control_time_based(
                    &mut pin,
                    activate_time,
                    deactivate_time,
                    cancel_token,
                    "light",
                )
                .await
            }
        }
    }
}
