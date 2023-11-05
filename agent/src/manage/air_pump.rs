use clap::Parser;

use super::{control::air_pump::AirPumpController, error::Error};

#[derive(Debug, Parser)]
pub struct AirPumpArgs {
    /// Whether to disable the air pump
    #[arg(
        id = "air_pump_disable",
        long = "air-pump-disable",
        env = "GROW_AGENT_AIR_PUMP_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to control the air pump
    #[arg(
        id = "air_pump_pin",
        long = "air-pump-pin",
        env = "GROW_AGENT_AIR_PUMP_PIN",
        default_value_t = 24
    )]
    pub pin: u8,
}

pub struct AirPumpManager {}

impl AirPumpManager {
    pub async fn start(args: AirPumpArgs) -> Result<(), Error> {
        AirPumpController::start(args.disable, args.pin)
            .await
            .map_err(Error::ControlError)
    }
}
