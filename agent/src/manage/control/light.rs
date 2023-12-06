use chrono::NaiveTime;
use clap::{Parser, ValueEnum};
use rppal::gpio::{Gpio, OutputPin};

use crate::error::AppError;

#[derive(Debug, Clone, ValueEnum)]
enum ControlMode {
    /// Disabled
    Disabled,
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
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_HOUR",
        default_value_t = NaiveTime::from_hms_opt(10, 0, 0).unwrap()
    )]
    activate_time: NaiveTime,

    /// The time of the day when the light should be switched off. Only has an effect in time mode
    #[arg(
        id = "light_control_switch_off_hour",
        long = "light-control-switch-off-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_HOUR",
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
    pub fn new(args: &LightControlArgs) -> Result<Self, AppError> {
        match args.mode {
            ControlMode::Disabled => Ok(Self::Disabled),
            ControlMode::Time => {
                let gpio = Gpio::new().map_err(AppError::InitGpioFailed)?;
                let pin = gpio
                    .get(args.pin)
                    .map_err(AppError::GetGpioFailed)?
                    .into_output();

                Ok(Self::Time {
                    pin,
                    activate_time: args.activate_time,
                    deactivate_time: args.deactivate_time,
                })
            }
        }
    }
}
