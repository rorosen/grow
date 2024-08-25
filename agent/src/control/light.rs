use std::path::Path;

use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};
use tokio_util::sync::CancellationToken;

use crate::config::light::{LightControlConfig, LightControlMode};

use super::{TimeBasedController, GPIO_CONSUMER, GPIO_LOW};

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
                    .request(LineRequestFlags::OUTPUT, GPIO_LOW, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;
                let controller =
                    TimeBasedController::new(handle, config.activate_time, config.deactivate_time)
                        .context("Failed to create time based controller")?;

                Ok(Self::TimeBased { controller })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            LightController::Disabled => Ok(()),
            LightController::TimeBased { mut controller } => {
                controller.run(cancel_token, "light").await
            }
        }
    }
}
