use std::time::Duration;

use clap::{Parser, ValueEnum};
use tokio_util::sync::CancellationToken;

use crate::{error::AppError, manage::control::control_cyclic};

use super::control::fan::FanController;

enum FanManager {
    DisabledControl,
    CyclicControl {
        controller: FanController,
        on_duration: Duration,
        off_duration: Duration,
    },
}

impl FanManager {
    pub async fn new(args: FanArgs) -> Result<Self, AppError> {
        match args.control_mode {
            ControlMode::Disabled => Ok(Self::DisabledControl),
            ControlMode::Cyclic => Ok(Self::CyclicControl {
                controller: FanController::new(args.pin)?,
                on_duration: Duration::from_secs(args.on_duration_secs),
                off_duration: Duration::from_secs(args.off_duration_secs),
            }),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting fan manager");

        match self {
            Self::DisabledControl => {
                log::info!("fan control is disabled");
                Ok(())
            }
            Self::CyclicControl {
                controller,
                on_duration,
                off_duration,
            } => control_cyclic(controller, on_duration, off_duration, cancel_token, "fan").await,
        }
    }
}
