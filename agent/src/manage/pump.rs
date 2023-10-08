use crate::{control::pump::PumpController, sample::water_level::WaterLevelSampler};

use super::Error;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct PumpArgs {
    /// Whether to disable the pump controller
    #[arg(
        id = "pump.disable",
        long = "pump-control-disable",
        env = "GROW_AGENT_PUMP_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to disable the left pump
    #[arg(
        id = "pump.left_pin",
        long = "pump-left-pin",
        env = "GROW_AGENT_PUMP_LEFT_PIN",
        default_value_t = 17
    )]
    pub left_pin: u8,

    /// The gpio pin used to disable the right pump
    #[arg(
        id = "pump.right_pin",
        long = "pump-right-pin",
        env = "GROW_AGENT_PUMP_RIGHT_PIN",
        default_value_t = 22
    )]
    pub right_pin: u8,

    pub lower_threshold: u16,

    pub upper_threshold: u16,
}

pub struct PumpManager {
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl PumpManager {
    pub async fn start() -> Result<(), Error> {
        Ok(())
    }

    async fn run() {}
}
