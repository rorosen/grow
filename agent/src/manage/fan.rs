use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{control::fan::FanController, error::Error};

#[derive(Debug, Parser)]
pub struct FanArgs {
    /// Whether to disable the circulation fan controller
    #[arg(
        id = "fan_control_disable",
        long = "fan-control-disable",
        env = "GROW_AGENT_FAN_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to control the circulation fans
    #[arg(
        id = "fan_control_pin",
        long = "fan-control-pin",
        env = "GROW_AGENT_FAN_CONTROL_PIN",
        default_value_t = 23
    )]
    pin: u8,

    /// The duration in seconds for which the circulation fans should run (0 means always stopped)
    #[arg(
        id = "fan_control_on_duration_secs",
        long = "fan-control-on-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    on_duration_secs: i64,

    /// The duration in seconds for which the circulation fans should be stopped (0 means always running)
    #[arg(
        id = "fan_control_off_duration_secs",
        long = "fan-control-off-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    off_duration_secs: i64,
}

pub struct FanManager {}

impl FanManager {
    pub async fn start(args: FanArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        FanController::start(
            cancel_token,
            args.disable,
            args.pin,
            args.on_duration_secs,
            args.off_duration_secs,
        )
        .await
        .map_err(Error::ControlError)
    }
}
