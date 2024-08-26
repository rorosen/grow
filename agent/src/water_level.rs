use std::path::Path;

use crate::{config::water_level::WaterLevelConfig, datastore::DataStore};

use super::{control::water_level::PumpController, sample::water_level::WaterLevelSampler};
use anyhow::{Context, Result};
use grow_measure::water_level::WaterLevelMeasurement;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct WaterLevelManager {
    receiver: mpsc::Receiver<Vec<WaterLevelMeasurement>>,
    controller: PumpController,
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
        let controller = PumpController::new(&config.control, &gpio_path)
            .context("Failed to initialize pump controller")?;
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
        log::debug!("Starting water manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("All water manager tasks finished");
                    return Ok(());
                }
                Some(measurements) = self.receiver.recv() => {
                    log::trace!("Water level measurements: {measurements:?}");
                    self.store.add_water_level_measurements(measurements)
                        .await
                        .context("Failed to save water level measurements")?;
                }
            }
        }
    }
}
