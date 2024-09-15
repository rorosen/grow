use anyhow::{Context, Result};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::fan::FanControlConfig;

use super::{Control, CyclicController, TimeBasedController};

pub struct FanController {
    inner: Option<Box<dyn Control + Send>>,
}

impl FanController {
    pub fn new(config: &FanControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        let controller: Option<Box<dyn Control + Send>> = match config {
            FanControlConfig::Off => None,
            FanControlConfig::Cyclic {
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
            FanControlConfig::TimeBased {
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
        const IDENTIFIER: &str = "Fan controller";

        if let Some(mut controller) = self.inner {
            log::info!("Starting fan controller");
            controller
                .run(cancel_token, IDENTIFIER)
                .await
                .context("Failed to run fan controller")?;
        } else {
            log::info!("Fan controller is disabled");
        }

        Ok(IDENTIFIER)
    }
}
