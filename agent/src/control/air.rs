use anyhow::{Context, Result};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

use crate::config::air::AirControlConfig;

use super::{Control, CyclicController, TimeBasedController};

pub struct AirController {
    inner: Option<Box<dyn Control + Send>>,
}

impl AirController {
    pub fn new(config: &AirControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        let controller: Option<Box<dyn Control + Send>> = match config {
            AirControlConfig::Off => None,
            AirControlConfig::Cyclic {
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
            AirControlConfig::TimeBased {
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
        const IDENTIFIER: &str = "Air controller";

        if let Some(mut controller) = self.inner {
            log::info!("Starting air controller");
            controller
                .run(cancel_token, IDENTIFIER)
                .await
                .context("Failed to run air controller")?;
        } else {
            log::info!("Air controller is disabled");
        }

        Ok(IDENTIFIER)
    }
}
