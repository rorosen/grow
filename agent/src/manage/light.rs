use super::control::light::LightController;
use crate::{error::AppError, manage::control::control_time_based};
use chrono::NaiveTime;
use clap::{Parser, ValueEnum};
use tokio_util::sync::CancellationToken;

enum LightManager {
    DisabledControl,
    TimeControl {
        controller: LightController,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    },
}

impl LightManager {
    pub async fn new(args: LightArgs) -> Result<Self, AppError> {
        match args.control_mode {
            ControlMode::Disabled => Ok(Self::DisabledControl),
            ControlMode::Time => Ok(Self::TimeControl {
                controller: LightController::new(args.pin)?,
                activate_time: args.activate_time,
                deactivate_time: args.deactivate_time,
            }),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting light manager");

        match self {
            Self::DisabledControl => {
                log::info!("light control is disabled");
                Ok(())
            }
            Self::TimeControl {
                controller,
                activate_time,
                deactivate_time,
            } => {
                control_time_based(
                    controller,
                    activate_time,
                    deactivate_time,
                    cancel_token,
                    "light",
                )
                .await
            }
        }
    }
}
