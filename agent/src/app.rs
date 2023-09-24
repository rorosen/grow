use crate::error::AppError;
use clap::Parser;
use log::LevelFilter;

#[derive(Debug, Parser)]
pub struct App {
    #[arg(short, long, env = "RUST_LOG", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[command(flatten)]
    exhaust_control_args: ExhaustControlArgs,

    #[command(flatten)]
    fan_control_args: FanControlArgs,

    #[command(flatten)]
    light_control_args: LightControlArgs,

    #[command(flatten)]
    pump_control_args: PumpControlArgs,
}

#[derive(Debug, Parser)]
pub struct ExhaustControlArgs {
    /// Whether to disable the exhaust fan controller
    #[arg(
        id = "exhaust_control.disable",
        long = "exhaust-control-disable",
        env = "GROW_AGENT_EXHAUST_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the exhaust fan in slow mode
    #[arg(
        id = "exhaust_control.pin_slow",
        long = "exhaust-control-pin-slow",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_SLOW",
        default_value_t = 25
    )]
    pin_slow: u8,

    /// The gpio pin used to disable the exhaust fan in fast mode (not implemented so far)
    #[arg(
        id = "exhaust_control.pin_fast",
        long = "exhaust-control-pin-fast",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_FAST",
        default_value_t = 26
    )]
    pin_fast: u8,

    /// The duration in seconds for which the exhaust fan should run (0 means always stopped)
    #[arg(
        id = "exhaust_control.on_duration_secs",
        long = "exhaust-control-on-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    on_duration_secs: u64,

    /// The duration in seconds for which the exhaust fan should be stopped (0 means always running)
    #[arg(
        id = "exhaust_control.off_duration_secs",
        long = "exhaust-control-off-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    off_duration_secs: u64,
}

#[derive(Debug, Parser)]
pub struct FanControlArgs {
    /// Whether to disable the circulation fan controller
    #[arg(
        id = "fan_control.disable",
        long = "fan-control-disable",
        env = "GROW_AGENT_FAN_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the left circulation fan
    #[arg(
        id = "fan_control.pin_left",
        long = "fan-control-pin-left",
        env = "GROW_AGENT_FAN_CONTROL_PIN_LEFT",
        default_value_t = 23
    )]
    pin_left: u8,

    /// The gpio pin used to disable the right circulation fan
    #[arg(
        id = "fan_control.pin_right",
        long = "fan-control-pin-right",
        env = "GROW_AGENT_FAN_CONTROL_PIN_RIGHT",
        default_value_t = 24
    )]
    pin_right: u8,

    /// The duration in seconds for which the circulation fans should run (0 means always stopped)
    #[arg(
        id = "fan_control.on_duration_secs",
        long = "fan-control-on-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    on_duration_secs: u64,

    /// The duration in seconds for which the circulation fans should be stopped (0 means always running)
    #[arg(
        id = "fan_control.off_duration_secs",
        long = "fan-control-off-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    off_duration_secs: u64,
}

#[derive(Debug, Parser)]
pub struct LightControlArgs {
    /// Whether to disable the light controller
    #[arg(
        id = "light_control.disable",
        long = "light-control-disable",
        env = "GROW_AGENT_LIGHT_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the light
    #[arg(
        id = "light_control.pin",
        long = "light-control-pin",
        env = "GROW_AGENT_LIGHT_CONTROL_PIN",
        default_value_t = 6
    )]
    pin: u8,

    /// The hour of the day (24h format) on which the light should be switched on
    #[arg(
        id = "light_control.switch_on_hour",
        long = "light-control-switch-on-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_HOUR",
        default_value_t = 10
    )]
    switch_on_hour: u8,

    /// The hour of the day (24h format) on which the light should be switched off
    #[arg(
        id = "light_control.switch_off_hour",
        long = "light-control-switch-off-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_HOUR",
        default_value_t = 22
    )]
    switch_off_hour: u8,
}

#[derive(Debug, Parser)]
pub struct PumpControlArgs {
    /// Whether to disable the pump controller
    #[arg(
        id = "pump_control.disable",
        long = "pump-control-disable",
        env = "GROW_AGENT_LIGHT_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the left pump
    #[arg(
        id = "pump_control.pin_left",
        long = "pump-control-pin-left",
        env = "GROW_AGENT_PUMP_CONTROL_LEFT_PIN",
        default_value_t = 17
    )]
    pin_left: u8,

    /// The gpio pin used to disable the right pump
    #[arg(
        id = "pump_control.pin_right",
        long = "pump-control-pin-right",
        env = "GROW_AGENT_PUMP_CONTROL_RIGHT_PIN",
        default_value_t = 22
    )]
    pin_right: u8,
    // tbd
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        Ok(())
    }
}
