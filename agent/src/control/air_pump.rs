use std::path::Path;

use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use tokio_util::sync::CancellationToken;

use crate::{
    config::air_pump::AirPumpControlConfig,
    control::{GPIO_ACTIVATE, GPIO_CONSUMER},
};

pub enum AirPumpController {
    Disabled,
    Permanent { handle: LineHandle },
}

impl AirPumpController {
    pub fn new(config: &AirPumpControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config {
            AirPumpControlConfig::Off => Ok(Self::Disabled),
            AirPumpControlConfig::AlwaysOn { pin } => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handle = chip
                    .get_line(*pin)
                    .context("Failed to get a handle to the GPIO line")?
                    .request(LineRequestFlags::OUTPUT, GPIO_ACTIVATE, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;

                Ok(Self::Permanent { handle })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Air pump controller";

        match self {
            Self::Disabled => {
                log::info!("Air pump controller is disabled");
                Ok(IDENTIFIER)
            }
            Self::Permanent { handle } => {
                log::info!("Air pump controller: Activating control pin");
                handle
                    .set_value(GPIO_ACTIVATE)
                    .context("Failed to set value of control pin")?;

                cancel_token.cancelled().await;
                Ok(IDENTIFIER)
            }
        }
    }
}
