use std::{path::Path, time::Duration};

use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::WaterLevelControlConfig;

use super::{Control, CyclicController, TimeBasedController};

pub struct WaterLevelController {
    inner: Option<Box<dyn Control + Send>>,
}

impl WaterLevelController {
    pub fn new(config: &WaterLevelControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        let controller: Option<Box<dyn Control + Send>> = match config {
            WaterLevelControlConfig::Off => None,
            WaterLevelControlConfig::Cyclic {
                pin,
                on_duration_secs,
                off_duration_secs,
            } => {
                let controller = Box::new(
                    CyclicController::new(
                        gpio_path,
                        *pin,
                        Duration::from_secs(*on_duration_secs),
                        Duration::from_secs(*off_duration_secs),
                    )
                    .context("Failed to create cyclic controller")?,
                );

                Some(controller)
            }
            WaterLevelControlConfig::TimeBased {
                pin,
                activate_time,
                deactivate_time,
            } => {
                let controller = Box::new(
                    TimeBasedController::new(gpio_path, *pin, *activate_time, *deactivate_time)
                        .context("Failed to create time based controller")?,
                );

                Some(controller)
            }
        };

        Ok(Self { inner: controller })
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Water level controller";

        if let Some(mut controller) = self.inner {
            log::info!("Starting air pump controller");
            controller
                .run(cancel_token, IDENTIFIER)
                .await
                .context("Failed to run air pump controller")?;
        } else {
            log::info!("Air pump controller is disabled");
        }

        Ok(IDENTIFIER)
    }
}
