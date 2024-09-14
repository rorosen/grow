use std::path::Path;

use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::WaterLevelControlConfig;

use super::{TimeBasedController, GPIO_CONSUMER, GPIO_DEACTIVATE};

pub enum WaterLevelController {
    Disabled,
    TimeBased { controller: TimeBasedController },
}

impl WaterLevelController {
    pub fn new(config: &WaterLevelControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config {
            WaterLevelControlConfig::Off => Ok(Self::Disabled),
            WaterLevelControlConfig::TimeBased {
                activate_time,
                deactivate_time,
                pumps,
            } => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handles: Result<Vec<LineHandle>> = pumps
                    .values()
                    .map(|pin| {
                        let handle = chip
                            .get_line(*pin)
                            .with_context(|| format!("Failed to get a handle to GPIO line {pin}"))?
                            .request(LineRequestFlags::OUTPUT, GPIO_DEACTIVATE, GPIO_CONSUMER)
                            .with_context(|| {
                                format!("Failed to get a handle to GPIO line {pin}")
                            })?;

                        Ok(handle)
                    })
                    .collect();

                let controller =
                    TimeBasedController::new(handles?, *activate_time, *deactivate_time)
                        .context("Failed to create time based controller")?;

                Ok(Self::TimeBased { controller })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Water level controller";

        match self {
            WaterLevelController::Disabled => {
                log::info!("Water level controller is disabled");
                Ok(IDENTIFIER)
            }
            WaterLevelController::TimeBased { mut controller } => {
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
