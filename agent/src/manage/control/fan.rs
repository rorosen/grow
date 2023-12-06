use crate::error::AppError;
use clap::{Parser, ValueEnum};
use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::control_cyclic;

#[derive(Debug, Clone, ValueEnum)]
enum ControlMode {
    /// Disabled control
    Disabled,
    /// Cyclic control
    Cyclic,
}

#[derive(Debug, Parser)]
pub struct FanControlArgs {
    /// The control mode
    #[arg(
        value_enum,
        id = "fan_control_mode",
        long = "fan-control-mode",
        env = "GROW_AGENT_FAN_CONTROL_MODE",
        default_value_t = ControlMode::Cyclic
    )]
    mode: ControlMode,

    /// The gpio pin used to control the circulation fans
    #[arg(
        id = "fan_control_pin",
        long = "fan-control-pin",
        env = "GROW_AGENT_FAN_CONTROL_PIN",
        default_value_t = 23
    )]
    pin: u8,

    /// The duration in seconds for which the circulation fans should
    /// run (0 means always stopped). Only has an effect in cyclic mode
    #[arg(
        id = "fan_control_on_duration_secs",
        long = "fan-control-on-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    on_duration_secs: u64,

    /// The duration in seconds for which the circulation fans should be
    /// stopped (0 means always running). Only has an effect in cyclic mode
    #[arg(
        id = "fan_control_off_duration_secs",
        long = "fan-control-off-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    off_duration_secs: u64,
}

pub enum FanController {
    Disabled,
    Cyclic {
        pin: OutputPin,
        on_duration: Duration,
        off_duration: Duration,
    },
}

impl FanController {
    pub fn new(args: &FanControlArgs) -> Result<Self, AppError> {
        match args.mode {
            ControlMode::Disabled => Ok(Self::Disabled),
            ControlMode::Cyclic => {
                let gpio = Gpio::new().map_err(AppError::InitGpioFailed)?;
                let pin = gpio
                    .get(args.pin)
                    .map_err(AppError::GetGpioFailed)?
                    .into_output();

                Ok(Self::Cyclic {
                    pin,
                    on_duration: Duration::from_secs(args.on_duration_secs),
                    off_duration: Duration::from_secs(args.off_duration_secs),
                })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<(), AppError> {
        match self {
            FanController::Disabled => Ok(()),
            FanController::Cyclic {
                mut pin,
                on_duration,
                off_duration,
            } => Ok(control_cyclic(
                &mut pin,
                on_duration,
                off_duration,
                cancel_token,
                "circulation fan",
            )
            .await),
        }
    }
}
