use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::fan::{FanControlConfig, FanControlMode};

use super::{CyclicController, GPIO_CONSUMER, GPIO_LOW};

pub enum FanController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl FanController {
    pub fn new(config: &FanControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            FanControlMode::Off => Ok(Self::Disabled),
            FanControlMode::Cyclic => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handle = chip
                    .get_line(config.pin)
                    .context("Failed to get a handle to the GPIO line")?
                    .request(LineRequestFlags::OUTPUT, GPIO_LOW, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;
                let controller = CyclicController::new(
                    handle,
                    Duration::from_secs(config.on_duration_secs),
                    Duration::from_secs(config.off_duration_secs),
                );

                Ok(Self::Cyclic { controller })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            FanController::Disabled => Ok(()),
            FanController::Cyclic { mut controller } => {
                controller.run(cancel_token, "circulation fan").await
            }
        }
    }
}
