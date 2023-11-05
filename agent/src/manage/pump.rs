use std::time::Duration;

use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{control::pump::PumpController, error::Error, sample::water_level::WaterLevelSampler};

#[derive(Debug, Parser)]
pub struct PumpArgs {
    /// Whether to disable the pump manager
    #[arg(
        id = "pump.disable",
        long = "pump-disable",
        env = "GROW_AGENT_PUMP_DISABLE"
    )]
    disable: bool,

    /// The gpio pin used to disable the left pump
    #[arg(
        id = "pump.control-pin-left",
        long = "pump-control-pin-left",
        env = "GROW_AGENT_PUMP_CONTROL_PIN_LEFT",
        default_value_t = 17
    )]
    pin_left: u8,

    /// The gpio pin used to disable the right pump
    #[arg(
        id = "pump.control-pin-right",
        long = "pump-control-pin-right",
        env = "GROW_AGENT_PUMP_CONTROL_PIN_RIGHT",
        default_value_t = 22
    )]
    pin_right: u8,

    /// The I2C address of the left water level sensor
    #[arg(
        id = "pump.sampler-addr-left",
        long = "pump-sampler-addr-left",
        env = "GROW_AGENT_WATER_LEVEL_ADDR_LEFT",
        default_value_t = 0x29
    )]
    sampler_addr_left: u8,

    /// The I2C address of the right water level sensor
    #[arg(
        id = "pump.sampler-addr-right",
        long = "pump-sampler-addr-left",
        env = "GROW_AGENT_WATER_LEVEL_ADDR_RIGHT",
        default_value_t = 0x29
    )]
    sampler_addr_right: u8,

    #[arg(
        id = "pump.sample-interval",
        long = "pump-sample-interval",
        env = "GROW_AGENT_WATER_LEVEL_SAMPLE_INTERVAL",
        default_value_t = 300
    )]
    sample_interval_sec: u64,
    // pub lower_threshold: u16,

    // pub upper_threshold: u16,
}

#[allow(dead_code)]
pub struct PumpManager {
    args: PumpArgs,
    controller: PumpController,
    sampler: Option<WaterLevelSampler>,
}

impl PumpManager {
    pub async fn start(args: PumpArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        if args.disable {
            log::info!("pump manager is disabled by configuration");
            return Ok(());
        }

        let controller =
            PumpController::new(args.pin_left, args.pin_right).map_err(Error::ControlError)?;

        let sampler =
            match WaterLevelSampler::new(args.sampler_addr_left, args.sampler_addr_right).await {
                Ok(s) => Some(s),
                Err(err) => {
                    log::warn!("failed to initialize water level sampler: {err}");
                    None
                }
            };

        Self {
            args,
            controller,
            sampler,
        }
        .run(cancel_token)
        .await
    }

    async fn run(mut self, cancel_token: CancellationToken) -> Result<(), Error> {
        let sample_interval = Duration::from_secs(self.args.sample_interval_sec);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping pump manager");
                    return Ok(());
                }
                _ = tokio::time::sleep(sample_interval) => {
                    log::debug!("taking sample of water level");
                    if self.sampler.is_none() {
                        match WaterLevelSampler::new(
                                self.args.sampler_addr_left,
                                self.args.sampler_addr_right).await {
                            Ok(sampler) => self.sampler = Some(sampler),
                            Err(err) => log::warn!("failed to initialize water level sampler: {err}"),
                        };
                    }

                    if let Some(ref mut sampler) = self.sampler {
                        match sampler.measure_range().await {
                            Ok(range) => {
                                log::info!("range is: {range} mm");
                            }
                            Err(err) => {
                                log::warn!("could not sample water level: {err}");
                            }
                        }
                    }
                }
            }
        }
    }
}
