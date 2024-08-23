use std::path::Path;

use crate::config::water_level::WaterLevelConfig;

use super::{
    control::pump::PumpController,
    sample::water_level::{WaterLevelSample, WaterLevelSampler},
};
use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct WaterLevelManager {
    receiver: mpsc::Receiver<WaterLevelSample>,
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl WaterLevelManager {
    pub async fn new(config: &WaterLevelConfig, i2c_path: impl AsRef<Path>) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller =
            PumpController::new(&config.control).context("Failed to initialize pump controller")?;
        let sampler = WaterLevelSampler::new(&config.sample, sender, &i2c_path)
            .await
            .context("Failed to initialize water level sampler")?;

        Ok(Self {
            receiver,
            controller,
            sampler,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("Starting water manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("All water manager tasks finished");
                    return;
                }
                Some(WaterLevelSample{measurements, ..}) = self.receiver.recv() => {
                    log::info!("Water level measurements: {measurements:?}");
                }
            }
        }
    }
}
