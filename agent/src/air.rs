use super::{control::air::AirController, sample::air::AirSampler};
use crate::{config::air::AirConfig, datastore::DataStore};
use anyhow::{Context, Result};
use grow_measure::air::AirMeasurement;
use std::path::Path;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

pub struct AirManager {
    receiver: mpsc::Receiver<Vec<AirMeasurement>>,
    controller: AirController,
    sampler: AirSampler,
    store: DataStore,
}

impl AirManager {
    pub async fn new(
        config: &AirConfig,
        store: DataStore,
        i2c_path: impl AsRef<Path>,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = AirController::new(&config.control, &gpio_path)
            .context("Failed to initialize exhaust fan controller")?;
        let sampler = AirSampler::new(&config.sample, sender, &i2c_path)
            .await
            .context("Failed to initialize air sampler")?;

        Ok(Self {
            receiver,
            controller,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<&'static str> {
        const IDENTIFIER: &str = "Air manager";

        let mut set = JoinSet::new();
        set.spawn(self.controller.run(cancel_token.clone()));
        set.spawn(self.sampler.run(cancel_token));

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            let id = ret
                                .context("Air manager task panicked")?
                                .context("Failed to run air manager task")?;
                            log::info!("{id} task terminated successfully");
                        },
                        None => return Ok(IDENTIFIER),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    log::trace!("Air measurements: {measurements:?}");
                    self.store.add_air_measurements(measurements).await?;
                }
            }
        }
    }
}
