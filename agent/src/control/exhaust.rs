use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::air::{ExhaustControlConfig, ExhaustControlMode};

use super::{CyclicController, GPIO_CONSUMER, GPIO_LOW};

pub enum ExhaustController {
    Disabled,
    Cyclic { controller: CyclicController },
}

impl ExhaustController {
    pub fn new(config: &ExhaustControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            ExhaustControlMode::Off => Ok(Self::Disabled),
            ExhaustControlMode::Cyclic => {
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
            ExhaustControlMode::Threshold => todo!(),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            ExhaustController::Disabled => Ok(()),
            ExhaustController::Cyclic { mut controller } => {
                controller.run(cancel_token, "exhaust fan").await
            }
        }
    }
}
