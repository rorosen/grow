use anyhow::{Context, Result};
use rppal::gpio::Gpio;
use tokio_util::sync::CancellationToken;

use crate::config::light::LightControlConfig;

use super::TimeBasedController;

pub enum LightController {
    Disabled,
    Time { controller: TimeBasedController },
}

impl LightController {
    pub fn new(config: &LightControlConfig) -> Result<Self> {
        if config.enable {
            let gpio = Gpio::new().context("Failed to initialize GPIO")?;
            let pin = gpio
                .get(config.pin)
                .with_context(|| format!("Failed to get gpio pin {}", config.pin))?
                .into_output();
            let controller =
                TimeBasedController::new(pin, config.activate_time, config.deactivate_time)
                    .context("Failed to create time based controller")?;

            Ok(Self::Time { controller })
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            LightController::Disabled => (),
            LightController::Time { mut controller } => controller.run(cancel_token, "light").await,
        }
    }
}
