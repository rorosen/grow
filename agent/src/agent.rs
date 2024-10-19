use std::{env, path::Path};

use crate::{
    air_manager::AirManager,
    config::{control::ControlConfig, Config},
    control::Controller,
    datastore::DataStore,
    light_sampler::LightSampler,
    water_level_manager::WaterLevelManager,
};
use anyhow::{anyhow, bail, Context, Result};
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
    cancel_token: CancellationToken,
    set: JoinSet<Result<()>>,
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

        Ok(Self {
            config,
            state_dir,
            cancel_token: CancellationToken::new(),
            set: JoinSet::new(),

        })
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

    pub async fn run(&mut self) -> Result<()> {
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
        self.set.spawn(
            air_manager
                .run(self.cancel_token.clone())
                .instrument(debug_span!("air manager")),
        );

        // if let Some(air_pump_controller) =
        //     Self::init_controller(&self.config.air_pump.control, &self.config.gpio_path)
        //         .context("Failed to initialize air pump controller")?
        // {
        //     self.set.spawn(
        //         air_pump_controller
        //             .run(self.cancel_token.clone())
        //             .instrument(debug_span!("air pump controller")),
        //     );
        // }

        // if let Some(fan_controller) =
        //     Self::init_controller(&self.config.fan.control, &self.config.gpio_path)
        //         .context("Failed to initialize fan controller")?
        // {
        //     self.set.spawn(
        //         fan_controller
        //             .run(self.cancel_token.clone())
        //             .instrument(debug_span!("fan controller")),
        //     );
        // }

        // if let Some(light_controller) =
        //     Self::init_controller(&self.config.light.control, &self.config.gpio_path)
        //         .context("Failed to initilaize light controller")?
        // {
        //     self.set.spawn(
        //         light_controller
        //             .run(self.cancel_token.clone())
        //             .instrument(debug_span!("light controller")),
        //     );
        // }

        // if let Some(light_sampler) = LightSampler::new(&self.config.light.sample, store.clone())
        //     .await
        //     .context("Failed to initilaize light sampler")?
        // {
        //     self.set.spawn(
        //         light_sampler
        //             .run(self.cancel_token.clone())
        //             .instrument(debug_span!("light sampler")),
        //     );
        // }
        //
        // let water_level_manager = WaterLevelManager::new(
        //     &self.config.water_level,
        //     store,
        //     &self.config.i2c_path,
        //     &self.config.gpio_path,
        // )
        // .await
        // .context("Failed to initialize water level manager")?;
        // self.set.spawn(
        //     water_level_manager
        //         .run(self.cancel_token.clone())
        //         .instrument(debug_span!("water level manager")),
        // );

        tokio::select! {
            _ = sigint.recv() => {
                info!("Shutting down on sigint...");
                self.shutdown().await
            }
            _ = sigterm.recv() => {
                info!("Shutting down on sigterm...");
                self.shutdown().await
            }
            opt = self.set.join_next() => {
                match opt {
                    Some(Ok(Ok(_))) => bail!("Task terminated unexpectedly"),
                    Some(Ok(Err(err))) => bail!("Task failed to run: {err:#}"),
                    Some(Err(err)) => bail!("Task panicked: {err:#}"),
                    None => {
                        bail!("All tasks terminated unexpectedly");
                    }
                }
            }
        }
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.cancel_token.cancel();
        let mut has_error = false;
        while let Some(res) = self.set.join_next().await {
            match res {
                Ok(Ok(_)) => (),
                Ok(Err(err)) => {
                    tracing::error!("Task failed during shutdown: {err:#}");
                    has_error = true;
                }
                Err(err) => {
                    tracing::error!("Task panicked during shutdown: {err:#}");
                    has_error = true;
                }
            }
        }

        if has_error {
            bail!("Errors occurred");
        }

        Ok(())
    }

    fn init_controller(
        config: &ControlConfig,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Option<Controller>> {
        match &config {
            ControlConfig::Off => None,
            ControlConfig::Cyclic {
                pin,
                on_duration: on_duration_secs,
                off_duration: off_duration_secs,
            } => Some(Controller::new_cyclic(
                gpio_path,
                *pin,
                *on_duration_secs,
                *off_duration_secs,
            )),
            ControlConfig::TimeBased {
                pin,
                activate_time,
                deactivate_time,
            } => Some(Controller::new_time_based(
                gpio_path,
                *pin,
                *activate_time,
                *deactivate_time,
            )),
            ControlConfig::Feedback { .. } => Some(Err(anyhow!(
                "Feedback control is not implemented for this type"
            ))),
        }
        .transpose()
    }
}
