use std::path::Path;

use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::config::water_level::{WaterLevelControlConfig, WaterLevelControlMode};

pub enum PumpController {
    Disabled,
}

impl PumpController {
    pub fn new(config: &WaterLevelControlConfig, _gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            WaterLevelControlMode::Off => Ok(Self::Disabled),
        }
    }

    pub async fn run(self, _: CancellationToken) -> Result<()> {
        match self {
            PumpController::Disabled => {
                log::info!("Pump controller is disabled");
                Ok(())
            }
        }
    }
}
