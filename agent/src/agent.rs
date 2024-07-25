use std::path::Path;

use crate::{
    config::Config,
    manage::{
        air::AirManager, light::LightManager, water::WaterLevelManager, AirPumpController,
        FanController,
    },
};
use anyhow::{bail, Context, Result};
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct Agent {
    config: Config,
}

impl Agent {
    pub fn new(config_path: impl AsRef<Path>) -> Result<Self> {
        let config = Config::from_path(config_path)?;
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        let mut sigint =
            signal(SignalKind::interrupt()).context("failed to register SIGINT handler")?;
        let mut sigterm =
            signal(SignalKind::terminate()).context("failed to register SIGTERM handler")?;

        AirPumpController::set_pin(&self.config.air_pump)
            .context("failed to configure air pump")?;
        let fan_controller =
            FanController::new(&self.config.fan).context("failed to initialize fan controller")?;
        let air_manager = AirManager::new(&self.config.air)
            .await
            .context("failed to initialize air manager")?;
        let light_manager = LightManager::new(&self.config.light)
            .await
            .context("failed to initialize light manager")?;
        let water_manager = WaterLevelManager::new(&self.config.water_level)
            .await
            .context("failed to initialize water manager")?;

        let mut set = JoinSet::new();
        let cancel_token = CancellationToken::new();
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
