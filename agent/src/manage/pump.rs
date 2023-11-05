use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{
    control::{
        air_pump::AirPumpControlArgs,
        pump::{PumpControlArgs, PumpController},
    },
    error::Error,
    sample::water_level::WaterLevelSampler,
};

#[derive(Debug, Parser)]
pub struct PumpArgs {
    /// Whether to disable the pump manager
    #[arg(
        id = "pump.disable",
        long = "pump-disable",
        env = "GROW_AGENT_PUMP_DISABLE"
    )]
    pub disable: bool,

    #[command(flatten)]
    control: PumpControlArgs,

    /// The I2C address of the left water level sensor
    #[arg(
        id = "pump.sampler-addr-left",
        long = "pump-sampler-addr-left",
        env = "GROW_AGENT_PUMP_SAMPLER_ADDR_LEFT",
        default_value_t = 0x29
    )]
    pub sampler_addr_left: u8,

    /// The I2C address of the right water level sensor
    #[arg(
        id = "pump.sampler-addr-right",
        long = "pump-sampler-addr-left",
        env = "GROW_AGENT_PUMP_SAMPLER_ADDR_RIGHT",
        default_value_t = 0x29
    )]
    pub sampler_addr_right: u8,

    pub lower_threshold: u16,

    pub upper_threshold: u16,

    #[command(flatten)]
    air_pump_control: AirPumpControlArgs,
}

#[allow(dead_code)]
pub struct PumpManager {
    controller: PumpController,
    sampler: Option<WaterLevelSampler>,
}

impl PumpManager {
    pub async fn start(args: PumpArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        if args.disable {
            log::info!("pump manager is disabled by configuration");
            return Ok(());
        }

        args.air_pump_control
            .set_air_pump()
            .map_err(Error::ControlError)?;
        let controller = PumpController::new(args.control).map_err(Error::ControlError)?;

        let sampler =
            match WaterLevelSampler::new(args.sampler_addr_left, args.sampler_addr_right).await {
                Ok(s) => Some(s),
                Err(err) => {
                    log::warn!("failed to initialize water level sampler: {err}");
                    None
                }
            };

        Self {
            controller,
            sampler,
        }
        .run(cancel_token)
        .await
    }

    async fn run(self, cancel_token: CancellationToken) -> Result<(), Error> {
        cancel_token.cancelled().await;
        Ok(())
    }
}
