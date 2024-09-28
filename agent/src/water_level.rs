use std::path::Path;

use crate::{config::water_level::WaterLevelConfig, control::Controller, datastore::DataStore};

use super::sample::water_level::WaterLevelSampler;
use anyhow::{Context, Result};
use grow_measure::water_level::WaterLevelMeasurement;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, trace, Instrument};

pub struct WaterLevelManager {
    receiver: mpsc::Receiver<Vec<WaterLevelMeasurement>>,
    controller: Controller,
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
        let controller = Controller::new(&config.control, &gpio_path)
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

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut set = JoinSet::new();
        set.spawn(
            self.controller
                .run(cancel_token.clone())
                .instrument(debug_span!("controller")),
        );
        set.spawn(self.sampler.run(cancel_token));

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            ret.context("Agent task panicked")?
                                .context("Failed to run agent task")?;
                        },
                        None => return Ok(()),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    trace!("Water level measurements: {measurements:?}");
                    self.store.add_water_level_measurements(measurements).await?
                }
            }
        }
    }
}
