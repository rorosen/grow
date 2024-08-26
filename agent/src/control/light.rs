use std::path::Path;

use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use tokio_util::sync::CancellationToken;

use crate::config::light::{LightControlConfig, LightControlMode};

use super::{TimeBasedController, GPIO_CONSUMER, GPIO_DEACTIVATE};

pub enum LightController {
    Disabled,
    TimeBased { controller: TimeBasedController },
}

impl LightController {
    pub fn new(config: &LightControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            LightControlMode::Off => Ok(Self::Disabled),
            LightControlMode::TimeBased => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handle = chip
                    .get_line(config.pin)
                    .context("Failed to get a handle to the GPIO line")?
                    .request(LineRequestFlags::OUTPUT, GPIO_DEACTIVATE, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;
                let controller =
                    TimeBasedController::new(handle, config.activate_time, config.deactivate_time)
                        .context("Failed to create time based controller")?;

                Ok(Self::TimeBased { controller })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Light controller";

        match self {
            LightController::Disabled => {
                log::info!("Light controller is disabled");
                Ok(IDENTIFIER)
            }
            LightController::TimeBased { mut controller } => {
                log::info!("Starting light controller");
                controller
                    .run(cancel_token, IDENTIFIER)
                    .await
                    .context("Failed to run light controller")?;
                Ok(IDENTIFIER)
            }
        }
    }
}
