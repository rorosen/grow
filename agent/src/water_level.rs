use std::path::Path;

use crate::{config::water_level::WaterLevelConfig, datastore::DataStore};

use super::{control::water_level::WaterLevelController, sample::water_level::WaterLevelSampler};
use anyhow::{Context, Result};
use grow_measure::water_level::WaterLevelMeasurement;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

pub struct WaterLevelManager {
    receiver: mpsc::Receiver<Vec<WaterLevelMeasurement>>,
    controller: WaterLevelController,
    sampler: WaterLevelSampler,
    store: DataStore,
}

impl WaterLevelManager {
    pub async fn new(
        config: &WaterLevelConfig,
        store: DataStore,
        i2c_path: impl AsRef<Path>,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = WaterLevelController::new(&config.control, &gpio_path)
            .context("Failed to initialize water level controller")?;
        let sampler = WaterLevelSampler::new(&config.sample, sender, &i2c_path)
            .await
            .context("Failed to initialize water level sampler")?;

        Ok(Self {
            receiver,
            controller,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Water level manager";

        let mut set = JoinSet::new();
        set.spawn(self.controller.run(cancel_token.clone()));
        set.spawn(self.sampler.run(cancel_token));

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            let id = ret
                                .context("Water level manager task panicked")?
                                .context("Failed to run water level manager task")?;
                            log::debug!("{id} task terminated successfully");
                        },
                        None => return Ok(IDENTIFIER),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    log::trace!("Water level measurements: {measurements:?}");
                    self.store.add_water_level_measurements(measurements).await?
                }
            }
        }
    }
}
