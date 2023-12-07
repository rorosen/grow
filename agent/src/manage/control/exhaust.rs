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
    /// Threshold control
    Threshold,
}

#[derive(Debug, Parser)]
pub struct ExhaustControlArgs {
    /// The control mode
    #[arg(
        value_enum,
        id = "air_control_mode",
        long = "air-control-mode",
        env = "GROW_AGENT_EXHAUST_CONTROL_MODE",
        default_value_t = ControlMode::Cyclic
    )]
    mode: ControlMode,

    /// The gpio pin used to enable the air fan
    #[arg(
        id = "air_control_pin",
        long = "air-control-pin",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN",
        default_value_t = 25
    )]
    pin: u8,

    /// The duration in seconds for which the air fan should
    /// run (0 means always stopped). Only has an effect in cyclic mode
    #[arg(
        id = "air_control_on_duration_secs",
        long = "air-control-on-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    on_duration_secs: u64,

    /// The duration in seconds for which the air fan should be
    /// stopped (0 means always running). Only has an effect in cyclic mode
    #[arg(
        id = "air_control_off_duration_secs",
        long = "air-control-off-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    off_duration_secs: u64,
}

pub enum ExhaustController {
    Disabled,
    Cyclic {
        pin: OutputPin,
        on_duration: Duration,
        off_duration: Duration,
    },
}

impl ExhaustController {
    pub fn new(args: &ExhaustControlArgs) -> Result<Self, AppError> {
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
            ControlMode::Threshold => todo!(),
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) {
        match self {
            ExhaustController::Disabled => (),
            ExhaustController::Cyclic {
                mut pin,
                on_duration,
                off_duration,
            } => control_cyclic(&mut pin, on_duration, off_duration, cancel_token, "exhaust").await,
        }
    }
}
