use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::fan::FanControlConfig;

use super::{CyclicController, GPIO_CONSUMER, GPIO_DEACTIVATE};

pub enum FanController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl FanController {
    pub fn new(config: &FanControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config {
            FanControlConfig::Off => Ok(Self::Disabled),
            FanControlConfig::Cyclic {
                pin,
                on_duration_secs,
                off_duration_secs,
            } => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handle = chip
                    .get_line(*pin)
                    .context("Failed to get a handle to the GPIO line")?
                    .request(LineRequestFlags::OUTPUT, GPIO_DEACTIVATE, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;
                let controller = CyclicController::new(
                    handle,
                    Duration::from_secs(*on_duration_secs),
                    Duration::from_secs(*off_duration_secs),
                );

                Ok(Self::Cyclic { controller })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Fan controller";

        match self {
            FanController::Disabled => {
                log::info!("Fan controller is disabled");
                Ok(IDENTIFIER)
            }
            FanController::Cyclic { mut controller } => {
                log::info!("Starting fan controller");
                controller
                    .run(cancel_token, IDENTIFIER)
                    .await
                    .context("Failed to run fan controller")?;
                Ok(IDENTIFIER)
            }
        }
    }
}
