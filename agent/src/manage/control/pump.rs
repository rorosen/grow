use anyhow::{Context, Result};
use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct PumpControlArgs {
    /// Whether to disable pump control
    #[arg(
        id = "pump_control_disable",
        long = "pump-control-disable",
        env = "GROW_AGENT_PUMP_CONTROL_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to control the left pump
    #[arg(
        id = "pump_control_left_pin",
        long = "pump-control-left-pin",
        env = "GROW_AGENT_PUMP_CONTROL_LEFT_PIN",
        default_value_t = 17
    )]
    left_pin: u8,

    /// The gpio pin used to control the right pump
    #[arg(
        id = "pump_control_right_pin",
        long = "pump-control-right-pin",
        env = "GROW_AGENT_PUMP_CONTROL_RIGHT_PIN",
        default_value_t = 22
    )]
    right_pin: u8,
}

pub enum PumpController {
    Enabled {
        left_pin: OutputPin,
        right_pin: OutputPin,
    },
    Disabled,
}

impl PumpController {
    pub fn new(args: &PumpControlArgs) -> Result<Self> {
        if args.disable {
            Ok(Self::Disabled)
        } else {
            let gpio = Gpio::new().context("failed to initialize GPIO")?;
            let left_pin = gpio
                .get(args.left_pin)
                .with_context(|| format!("failed to get gpio pin {} (left)", args.left_pin))?
                .into_output();
            let right_pin = gpio
                .get(args.right_pin)
                .with_context(|| format!("failed to get gpio pin {} (right)", args.right_pin))?
                .into_output();

            Ok(Self::Enabled {
                left_pin,
                right_pin,
            })
        }
    }

    pub async fn run(self, _: CancellationToken) {}
}
