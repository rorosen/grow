use crate::config::water_level::WaterLevelConfig;

use super::{control::pump::PumpController, sample::water_level::WaterLevelSampler};
use anyhow::{Context, Result};
use grow_measure::WaterLevelMeasurement;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct WaterLevelManager {
    receiver: mpsc::Receiver<WaterLevelMeasurement>,
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl WaterLevelManager {
    pub fn new(config: &WaterLevelConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller =
            PumpController::new(&config.control).context("failed to initialize pump controller")?;

        Ok(Self {
            receiver,
            controller,
            sampler: WaterLevelSampler::new(&config.sample, sender),
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("starting water manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("all water manager tasks finished");
                    return;
                }
                Some(measurement) = self.receiver.recv() => {
                    log::info!("water level measurement: {measurement:?}");
                }
            }
        }
    }
}
