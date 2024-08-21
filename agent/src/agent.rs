use std::path::Path;

use crate::{
    air::AirManager,
    config::Config,
    control::{air_pump::AirPumpController, fan::FanController},
    light::LightManager,
    water::WaterLevelManager,
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
            signal(SignalKind::interrupt()).context("Failed to register SIGINT handler")?;
        let mut sigterm =
            signal(SignalKind::terminate()).context("Failed to register SIGTERM handler")?;

        AirPumpController::set_pin(&self.config.air_pump)
            .context("Failed to configure air pump")?;
        let fan_controller =
            FanController::new(&self.config.fan).context("Failed to initialize fan controller")?;
        let air_manager = AirManager::new(&self.config.air)
            .await
            .context("Failed to initialize air manager")?;
        let light_manager = LightManager::new(&self.config.light)
            .await
            .context("Failed to initialize light manager")?;
        let water_manager = WaterLevelManager::new(&self.config.water_level)
            .await
            .context("Failed to initialize water level manager")?;

        let mut set = JoinSet::new();
        let cancel_token = CancellationToken::new();
        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("Fan controller", fan_controller.run(cloned_token).await) });
        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("Air manager", air_manager.run(cloned_token).await) });
        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("Light manager", light_manager.run(cloned_token).await) });
        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("Water manager", water_manager.run(cloned_token).await) });

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    log::info!("Shutting down on sigint");
                    cancel_token.cancel();
                }
                _ = sigterm.recv() => {
                    log::info!("Shutting down on sigterm");
                    cancel_token.cancel();
                }
                res = set.join_next() => {
                    match res {
                        Some(Ok((id, _))) => log::info!("{id} task terminated"),
                        Some(Err(err)) => {
                            bail!("Task panicked: {err:#}");
                        }
                        None => {
                            log::info!("All tasks terminated");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
