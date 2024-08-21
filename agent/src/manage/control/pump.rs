use std::collections::HashMap;

use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use crate::config::water_level::PumpControlConfig;

pub enum PumpController {
    Enabled {
        #[allow(dead_code)]
        pins: HashMap<String, OutputPin>,
    },
    Disabled,
}

impl PumpController {
    pub fn new(config: &PumpControlConfig) -> Result<Self> {
        if config.enable {
            let mut pins = HashMap::new();
            let gpio = Gpio::new().context("Failed to initialize GPIO")?;
            for (identifier, pin) in &config.pumps {
                let pin = gpio
                    .get(*pin)
                    .with_context(|| {
                        format!("Failed to get gpio pin {pin} of {identifier} water pump")
                    })?
                    .into_output();

                pins.insert(identifier.into(), pin);
            }

            Ok(Self::Enabled { pins })
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn run(self, _: CancellationToken) {
        match self {
            PumpController::Enabled { .. } => todo!("Implement pump controller run"),
            PumpController::Disabled => (),
        }
    }
}
