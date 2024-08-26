use std::path::Path;

use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::{WaterLevelControlConfig, WaterLevelControlMode};

pub enum WaterLevelController {
    Disabled,
}

impl WaterLevelController {
    pub fn new(config: &WaterLevelControlConfig, _gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            WaterLevelControlMode::Off => Ok(Self::Disabled),
        }
    }

    pub async fn run(self, _: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Water level controller";

        match self {
            WaterLevelController::Disabled => {
                log::info!("Water level controller is disabled");
                Ok(IDENTIFIER)
            }
        }
    }
}
