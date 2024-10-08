use std::path::Path;

use crate::{
    config::{
        control::ControlConfig,
        water_level::{WaterLevelConfig, WaterLevelSensorConfig, WaterLevelSensorModel},
    },
    control::Controller,
    datastore::DataStore,
    measure::{vl53l0x::Vl53L0X, WaterLevelMeasurement},
    sample::Sampler,
};

use anyhow::{bail, Context, Result};
use futures::future::join_all;
use tokio::{sync::broadcast::{self, error::RecvError}, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, warn, Instrument};

pub struct WaterLevelManager {
    controller: Controller,
    receiver: broadcast::Receiver<Vec<WaterLevelMeasurement>>,
    sampler: Sampler<Vl53L0X>,
    store: DataStore,
}

impl WaterLevelManager {
    pub async fn new(
        config: &WaterLevelConfig,
        store: DataStore,
        i2c_path: &Path,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let (sender, receiver) = broadcast::channel(8);
        let controller = match &config.control {
            ControlConfig::Off => Controller::new_disabled(),
            ControlConfig::Cyclic {
                pin,
                on_duration_secs,
                off_duration_secs,
            } => Controller::new_cyclic(gpio_path, *pin, *on_duration_secs, *off_duration_secs)?,
            ControlConfig::TimeBased {
                pin,
                activate_time,
                deactivate_time,
            } => Controller::new_time_based(gpio_path, *pin, *activate_time, *deactivate_time)?,
            ControlConfig::Feedback {
                pin,
                activate_condition,
                deactivate_condition,
            } => {
                if config.sample.sensors.is_empty() {
                    bail!("Feedback control requires at least one activated water level sensor");
                }

                Controller::new_threshold(
                activate_condition,
                deactivate_condition,
                gpio_path,
                *pin,
                sender.subscribe(),
            )?
            },
        };

        let sensors = join_all(
            config
                .sample
                .sensors
                .iter()
                .map(|(label, config)| Self::init_sensor(config, label, i2c_path)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Vl53L0X>>>()?;

        let sampler = Sampler::new(config.sample.sample_rate_secs, sender, sensors)
            .context("Failed to initialize water level sampler")?;

        Ok(Self {
            controller,
            receiver,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut set = JoinSet::new();
        set.spawn(
            self.controller
                .run(cancel_token.clone())
                .instrument(debug_span!("controller")),
        );
        set.spawn(
            self.sampler
                .run(cancel_token.clone())
                .instrument(debug_span!("sampler")),
        );

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            ret.context("Water level task panicked")?
                                .context("Failed to run water level task")?;
                        },
                        None => return Ok(()),
                    }
                }
                res = self.receiver.recv() => {
                    match res {
                        Ok(measurements) => {
                            self.store
                                .add_water_level_measurements(measurements)
                                .await
                                .context("Failed to store water level measurements")?;
                        },
                        Err(RecvError::Lagged(num_skipped)) => warn!("Skipping {num_skipped} measurements due to lagging"),
                        Err(RecvError::Closed) => {
                            if !cancel_token.is_cancelled() {
                                bail!("Failed to receive measurements: {}", RecvError::Closed)
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init_sensor(
        config: &WaterLevelSensorConfig,
        label: &str,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Vl53L0X> {
        match config.model {
            WaterLevelSensorModel::Vl53L0X => {
                Vl53L0X::new(i2c_path, config.address, label.to_owned())
                    .await
                    .with_context(|| format!("Failed to initialize {:?} water level sensor", label))
            }
        }
    }
}
