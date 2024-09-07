use std::env;

use crate::{
    air::AirManager,
    config::Config,
    control::{air_pump::AirPumpController, fan::FanController},
    datastore::DataStore,
    light::LightManager,
    water_level::WaterLevelManager,
};
use anyhow::{Context, Result};
use tokio::{
    signal::unix::{signal, SignalKind},
    task::{spawn_blocking, JoinSet},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct Agent {
    config: Config,
    state_dir: String,
}

impl Agent {
    pub async fn new() -> Result<Self> {
        let state_dir =
            Self::get_systemd_dir("STATE_DIRECTORY").context("Failed to get state directory")?;
        let config_path = env::var("GROW_AGENT_CONFIG_PATH")
            .or_else(|_| -> Result<String> {
                let config_dir = Self::get_systemd_dir("CONFIGURATION_DIRECTORY")?;
                Ok(format!("{config_dir}/config.json"))
            })
            .context("failed to get config path")?;

        let config = spawn_blocking(move || {
            Config::from_file(&config_path)
                .with_context(|| format!("Failed to initialize config from {config_path}"))
        })
        .await
        .context("Panic while initializing config")??;

        Ok(Self { config, state_dir })
    }

    fn get_systemd_dir(name: &str) -> Result<String> {
        let dirs =
            env::var(name).with_context(|| format!("Failed to read {name} from environment"))?;
        let dir = dirs
            .split(':')
            .next()
            .with_context(|| format!("Failed to get directory from {dirs:?}"))?
            .to_string();

        Ok(dir)
    }

    pub async fn run(self) -> Result<()> {
        let mut sigint =
            signal(SignalKind::interrupt()).context("Failed to register SIGINT handler")?;
        let mut sigterm =
            signal(SignalKind::terminate()).context("Failed to register SIGTERM handler")?;

        let store = DataStore::new(&format!(
            "sqlite://{}/{}.sqlite3",
            self.state_dir, self.config.grow_id
        ))
        .await
        .context("Failed to initialize data store")?;

        let air_pump_controller =
            AirPumpController::new(&self.config.air_pump_control, &self.config.gpio_path)
                .context("Failed to initialize air pump controller")?;
        let fan_controller = FanController::new(&self.config.fan, &self.config.gpio_path)
            .context("Failed to initialize fan controller")?;

        let air_manager = AirManager::new(
            &self.config.air,
            store.clone(),
            &self.config.i2c_path,
            &self.config.gpio_path,
        )
        .await
        .context("Failed to initialize air manager")?;

        let light_manager = LightManager::new(
            &self.config.light,
            store.clone(),
            &self.config.i2c_path,
            &self.config.gpio_path,
        )
        .await
        .context("Failed to initialize light manager")?;

        let water_manager = WaterLevelManager::new(
            &self.config.water_level,
            store,
            &self.config.i2c_path,
            &self.config.gpio_path,
        )
        .await
        .context("Failed to initialize water level manager")?;

        let cancel_token = CancellationToken::new();
        let mut set = JoinSet::new();
        set.spawn(air_manager.run(cancel_token.clone()));
        set.spawn(air_pump_controller.run());
        set.spawn(fan_controller.run(cancel_token.clone()));
        set.spawn(light_manager.run(cancel_token.clone()));
        set.spawn(water_manager.run(cancel_token.clone()));

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
                        Some(ret) => {
                            let id = ret
                                .context("Agent task panicked")?
                                .context("Failed to run agent task")?;
                            log::info!("{id} task terminated successfully");
                        },
                        None => {
                            log::info!("All tasks terminated successfully");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}
