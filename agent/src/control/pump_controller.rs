use clap::Parser;

#[derive(Debug, Parser)]
pub struct PumpControlArgs {
    /// Whether to disable the pump controller
    #[arg(
        id = "pump_control.disable",
        long = "pump-control-disable",
        env = "GROW_AGENT_PUMP_CONTROL_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to disable the left pump
    #[arg(
        id = "pump_control.pin_left",
        long = "pump-control-pin-left",
        env = "GROW_AGENT_PUMP_CONTROL_LEFT_PIN",
        default_value_t = 17
    )]
    pub pin_left: u8,

    /// The gpio pin used to disable the right pump
    #[arg(
        id = "pump_control.pin_right",
        long = "pump-control-pin-right",
        env = "GROW_AGENT_PUMP_CONTROL_RIGHT_PIN",
        default_value_t = 22
    )]
    pub pin_right: u8,
    // tbd
}
