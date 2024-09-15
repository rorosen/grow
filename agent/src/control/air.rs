use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::air::AirControlConfig;

use super::{CyclicController, GPIO_CONSUMER, GPIO_DEACTIVATE};

pub enum AirController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl AirController {
    pub fn new(config: &AirControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config {
            AirControlConfig::Off => Ok(Self::Disabled),
            AirControlConfig::Cyclic {
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
        const IDENTIFIER: &str = "Air controller";

        match self {
            AirController::Disabled => {
                log::info!("Air controller is disabled");
                Ok(IDENTIFIER)
            }
            AirController::Cyclic { mut controller } => {
                log::info!("Starting air controller");
                controller
                    .run(cancel_token, IDENTIFIER)
                    .await
                    .context("Failed to run air controller")?;
                Ok(IDENTIFIER)
            }
        }
    }
}
