use std::path::Path;

use anyhow::{Context, Result};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use tokio_util::sync::CancellationToken;

use crate::{
    config::air_pump::{AirPumpControlConfig, AirPumpControlMode},
    control::{GPIO_CONSUMER, GPIO_HIGH},
};

pub enum AirPumpController {
    Disabled,
    Permanent { handle: LineHandle },
}

impl AirPumpController {
    pub fn new(config: &AirPumpControlConfig, gpio_path: impl AsRef<Path>) -> Result<Self> {
        match config.mode {
            AirPumpControlMode::Off => Ok(Self::Disabled),
            AirPumpControlMode::Permanent => {
                let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
                let handle = chip
                    .get_line(config.pin)
                    .context("Failed to get a handle to the GPIO line")?
                    .request(LineRequestFlags::OUTPUT, GPIO_HIGH, GPIO_CONSUMER)
                    .context("Failed to get access to the GPIO")?;

                Ok(Self::Permanent { handle })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            Self::Disabled => {
                cancel_token.cancelled().await;
                Ok(())
            }
            Self::Permanent { handle } => {
                handle
                    .set_value(GPIO_HIGH)
                    .context("Failed to set value of control pin")?;
                cancel_token.cancelled().await;
                Ok(())
            }
        }
    }
}
