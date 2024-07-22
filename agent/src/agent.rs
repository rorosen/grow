use crate::manage::{
    air::{AirArgs, AirManager},
    light::{LightArgs, LightManager},
    water::{WaterArgs, WaterManager},
    AirPumpControlArgs, AirPumpController, FanControlArgs, FanController,
};
use anyhow::{bail, Context, Result};
use clap::Parser;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct Agent {
    #[command(flatten)]
    light_args: LightArgs,

    #[command(flatten)]
    water_args: WaterArgs,

    #[command(flatten)]
    fan_args: FanControlArgs,

    #[command(flatten)]
    air_args: AirArgs,

    #[command(flatten)]
    air_pump_args: AirPumpControlArgs,
}

impl Agent {
    pub async fn run(self) -> Result<()> {
        let mut sigint =
            signal(SignalKind::interrupt()).context("failed to register SIGINT handler")?;
        let mut sigterm =
            signal(SignalKind::terminate()).context("failed to register SIGTERM handler")?;
        let cancel_token = CancellationToken::new();

        let fan_controller =
            FanController::new(&self.fan_args).context("failed to initialize fan controller")?;

        let air_manager = AirManager::new(&self.air_args)
            .await
            .context("failed to initialize air manager")?;

        let light_manager = LightManager::new(&self.light_args)
            .await
            .context("failed to initialize light manager")?;

        let water_manager =
            WaterManager::new(&self.water_args).context("failed to initialize water manager")?;

        AirPumpController::set(&self.air_pump_args).context("failed to configure air pump")?;

        let mut set = JoinSet::new();

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("fan controller", fan_controller.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("air manager", air_manager.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("light manager", light_manager.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("water manager", water_manager.run(cloned_token).await) });

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    log::info!("shutting down on sigint");
                    cancel_token.cancel();
                }
                _ = sigterm.recv() => {
                    log::info!("shutting down on sigterm");
                    cancel_token.cancel();
                }
                res = set.join_next() => {
                    match res {
                        Some(Ok((id, _))) => log::info!("{id} task terminated"),
                        Some(Err(err)) => {
                            bail!("task panicked: {err:#}");
                        }
                        None => {
                            log::info!("all manager tasks terminated");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
