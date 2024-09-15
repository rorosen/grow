use std::path::Path;

use crate::{config::light::LightConfig, datastore::DataStore};

use super::{control::light::LightController, sample::light::LightSampler};
use anyhow::{Context, Result};
use grow_measure::light::LightMeasurement;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

pub struct LightManager {
    receiver: mpsc::Receiver<Vec<LightMeasurement>>,
    controller: LightController,
    sampler: LightSampler,
    store: DataStore,
}

impl LightManager {
    pub async fn new(
        config: &LightConfig,
        store: DataStore,
        i2c_path: impl AsRef<Path>,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = LightController::new(&config.control, &gpio_path)
            .context("Failed to initialize light controller")?;
        let sampler = LightSampler::new(&config.sample, sender, &i2c_path)
            .await
            .context("Failed to initialize light sampler")?;

        Ok(Self {
            receiver,
            controller,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Light manager";

        let mut set = JoinSet::new();
        set.spawn(self.controller.run(cancel_token.clone()));
        set.spawn(self.sampler.run(cancel_token));

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            let id = ret
                                .context("Light manager task panicked")?
                                .context("Failed to run light manager task")?;
                            log::debug!("{id} task terminated successfully");
                        },
                        None => return Ok(IDENTIFIER),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    log::trace!("Light measurements: {measurements:?}");
                    self.store.add_light_measurements(measurements).await?;
                }
            }
        }
    }
}
