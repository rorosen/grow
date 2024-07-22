use anyhow::{Context, Result};
use chrono::NaiveTime;
use clap::{Parser, ValueEnum};
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use super::control_time_based;

#[derive(Debug, Clone, ValueEnum)]
enum ControlMode {
    /// Disabled
    Off,
    /// Time-based control
    Time,
}

#[derive(Debug, Parser)]
pub struct LightControlArgs {
    /// The control mode
    #[arg(
        value_enum,
        id = "light_control_mode",
        long = "light-control-mode",
        env = "GROW_AGENT_LIGHT_CONTROL_MODE",
        default_value_t = ControlMode::Time
    )]
    mode: ControlMode,

    /// The gpio pin used to disable the light
    #[arg(
        id = "light_control_pin",
        long = "light-control-pin",
        env = "GROW_AGENT_LIGHT_CONTROL_PIN",
        default_value_t = 6
    )]
    pin: u8,

    /// The time of the day when the light should be switched on. Only has an effect in time mode
    #[arg(
        id = "light_control_switch_on_hour",
        long = "light-control-switch-on-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_TIME",
        default_value_t = NaiveTime::from_hms_opt(10, 0, 0).unwrap()
    )]
    activate_time: NaiveTime,

    /// The time of the day when the light should be switched off. Only has an effect in time mode
    #[arg(
        id = "light_control_switch_off_hour",
        long = "light-control-switch-off-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_TIME",
        default_value_t = NaiveTime::from_hms_opt(22, 0, 0).unwrap()
    )]
    deactivate_time: NaiveTime,
}

pub enum LightController {
    Disabled,
    Time {
        pin: OutputPin,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    },
}

impl LightController {
    pub fn new(args: &LightControlArgs) -> Result<Self> {
        match args.mode {
            ControlMode::Off => Ok(Self::Disabled),
            ControlMode::Time => {
                let gpio = Gpio::new().context("failed to initialize GPIO")?;
                let pin = gpio
                    .get(args.pin)
                    .with_context(|| format!("failed to get gpio pin {}", args.pin))?
                    .into_output();

                Ok(Self::Time {
                    pin,
                    activate_time: args.activate_time,
                    deactivate_time: args.deactivate_time,
                })
            }
        }
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        match self {
            LightController::Disabled => Ok(()),
            LightController::Time {
                mut pin,
                activate_time,
                deactivate_time,
            } => {
                control_time_based(
                    &mut pin,
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
