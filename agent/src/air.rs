use super::sample::air::AirSampler;
use crate::{config::air::AirConfig, control::Controller, datastore::DataStore};
use anyhow::{Context, Result};
use grow_measure::air::AirMeasurement;
use tracing::trace;
use std::path::Path;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

pub struct AirManager {
    receiver: mpsc::Receiver<Vec<AirMeasurement>>,
    controller: Controller,
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
        let controller = Controller::new(&config.control, &gpio_path)
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

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut set = JoinSet::new();
        set.spawn(self.controller.run(cancel_token.clone()));
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
                    trace!("Air measurements: {measurements:?}");
                    self.store.add_air_measurements(measurements).await?;
                }
            }
        }
    }
}
