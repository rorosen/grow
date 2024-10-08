use std::{env, path::Path};

use crate::{
    air_manager::AirManager,
    config::{control::ControlConfig, Config},
    control::Controller,
    datastore::DataStore,
    light_sampler::LightSampler,
    water_level_manager::WaterLevelManager,
};
use anyhow::{bail, Context, Result};
use tokio::{
    signal::unix::{signal, SignalKind},
    task::{spawn_blocking, JoinSet},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, info, Instrument as _};

#[derive(Debug)]
pub struct Agent {
    config: Config,
    state_dir: String,
}

impl Agent {
    pub async fn new() -> Result<Self> {
        let state_dir =
            Self::first_systemd_dir("STATE_DIRECTORY").context("Failed to get state directory")?;
        let config_path = env::var("GROW_AGENT_CONFIG_PATH")
            .or_else(|_| -> Result<String> {
                let config_dir = Self::first_systemd_dir("CONFIGURATION_DIRECTORY")?;
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

    fn first_systemd_dir(name: &str) -> Result<String> {
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

        let air_manager = AirManager::new(
            &self.config.air,
            store.clone(),
            &self.config.i2c_path,
            &self.config.gpio_path,
        )
        .await
        .context("Failed to initialize air manager")?;

        let air_pump_controller =
            Self::init_controller(&self.config.air_pump.control, &self.config.gpio_path)
                .context("Failed to initialize air pump controller")?;

        let fan_controller =
            Self::init_controller(&self.config.fan.control, &self.config.gpio_path)
                .context("Failed to initialize fan controller")?;

        let light_controller =
            Self::init_controller(&self.config.light.control, &self.config.gpio_path)
                .context("Failed to initilaize light controller")?;

        let light_sampler = LightSampler::new(
            &self.config.light.sample,
            &self.config.i2c_path,
            store.clone(),
        )
        .await
        .context("Failed to initilaize light sampler")?;

        let water_level_manager = WaterLevelManager::new(
            &self.config.water_level,
            store,
            &self.config.i2c_path,
            &self.config.gpio_path,
        )
        .await
        .context("Failed to initialize water level manager")?;

        let cancel_token = CancellationToken::new();
        let mut set = JoinSet::new();
        set.spawn(
            air_manager
                .run(cancel_token.clone())
                .instrument(debug_span!("air manager")),
        );
        set.spawn(
            air_pump_controller
                .run(cancel_token.clone())
                .instrument(debug_span!("air pump controller")),
        );
        set.spawn(
            fan_controller
                .run(cancel_token.clone())
                .instrument(debug_span!("fan controller")),
        );
        set.spawn(
            light_controller
                .run(cancel_token.clone())
                .instrument(debug_span!("light controller")),
        );
        set.spawn(
            light_sampler
                .run(cancel_token.clone())
                .instrument(debug_span!("light sampler")),
        );
        set.spawn(
            water_level_manager
                .run(cancel_token.clone())
                .instrument(debug_span!("water level manager")),
        );

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    info!("Shutting down on sigint...");
                    cancel_token.cancel();
                }
                _ = sigterm.recv() => {
                    info!("Shutting down on sigterm...");
                    cancel_token.cancel();
                }
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            ret.context("Task panicked")?
                                .context("Failed to run task")?;
                        },
                        None => {
                            info!("All tasks terminated successfully");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    fn init_controller(config: &ControlConfig, gpio_path: impl AsRef<Path>) -> Result<Controller> {
        match &config {
            ControlConfig::Off => Ok(Controller::new_disabled()),
            ControlConfig::Cyclic {
                pin,
                on_duration_secs,
                off_duration_secs,
            } => Controller::new_cyclic(gpio_path, *pin, *on_duration_secs, *off_duration_secs),
            ControlConfig::TimeBased {
                pin,
                activate_time,
                deactivate_time,
            } => Controller::new_time_based(gpio_path, *pin, *activate_time, *deactivate_time),
            ControlConfig::Feedback { .. } => {
                bail!("Feedback control is not implement for this type")
            }
        }
    }
}
