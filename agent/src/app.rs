use clap::Parser;
use log::LevelFilter;

use crate::error::AppError;

#[derive(Debug, Parser)]
pub struct App {
    #[arg(short, long, env = "RUST_LOG", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[command(flatten)]
    pin: PinArgs,
}

#[derive(Debug, Parser)]
pub struct PinArgs {
    /// The gpio pin used to enable the exhaust fan in slow mode
    #[arg(long, env = "GROW_AGENT_EXHAUST_SLOW_PIN", default_value_t = 25)]
    exhaust_slow: u8,

    /// The gpio pin used to enable the exhaust fan in fast mode
    #[arg(long, env = "GROW_AGENT_EXHAUST_FAST_PIN", default_value_t = 26)]
    exhaust_fast: u8,

    /// The gpio pin used to enable the left circulation fan
    #[arg(long, env = "GROW_AGENT_FAN_LEFT_PIN", default_value_t = 23)]
    fan_left: u8,

    /// The gpio pin used to enable the right circulation fan
    #[arg(long, env = "GROW_AGENT_FAN_RIGHT_PIN", default_value_t = 24)]
    fan_right: u8,

    /// The gpio pin used to enable the light
    #[arg(long, env = "GROW_AGENT_LIGHT_PIN", default_value_t = 6)]
    light: u8,

    /// The gpio pin used to enable the left pump
    #[arg(long, env = "GROW_AGENT_PUMP_LEFT_PIN", default_value_t = 17)]
    pump_left: u8,

    /// The gpio pin used to enable the right pump
    #[arg(long, env = "GROW_AGENT_PUMP_RIGHT_PIN", default_value_t = 22)]
    pump_right: u8,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        println!("{:?}", self);
        Ok(())
    }
}
