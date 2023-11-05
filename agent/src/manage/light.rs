use chrono::NaiveTime;
use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{control::light::LightController, error::Error};

#[derive(Debug, Parser)]
pub struct LightArgs {
    /// Whether to disable the light controller
    #[arg(
        id = "light_control_disable",
        long = "light-control-disable",
        env = "GROW_AGENT_LIGHT_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the light
    #[arg(
        id = "light_control_pin",
        long = "light-control-pin",
        env = "GROW_AGENT_LIGHT_CONTROL_PIN",
        default_value_t = 6
    )]
    pin: u8,

    /// The time of the day when the light should be switched on
    #[arg(
        id = "light_control_switch_on_hour",
        long = "light-control-switch-on-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_HOUR",
        default_value_t = NaiveTime::from_hms_opt(10, 0, 0).unwrap()
    )]
    activate_time: NaiveTime,

    /// The time of the day when the light should be switched off
    #[arg(
        id = "light_control_switch_off_hour",
        long = "light-control-switch-off-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_HOUR",
        default_value_t = NaiveTime::from_hms_opt(22, 0, 0).unwrap()
    )]
    deactivate_time: NaiveTime,
}

pub struct LightManager {}

impl LightManager {
    pub async fn start(args: LightArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        LightController::start(
            cancel_token,
            args.disable,
            args.pin,
            args.activate_time,
            args.deactivate_time,
        )
        .await
        .map_err(Error::ControlError)
    }
}
