use std::path::Path;

use crate::config::light::LightConfig;

use super::{control::light::LightController, sample::light::LightSampler};
use anyhow::{Context, Result};
use grow_measure::light::LightMeasurement;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct LightManager {
    receiver: mpsc::Receiver<Vec<LightMeasurement>>,
    controller: LightController,
    sampler: LightSampler,
}

impl LightManager {
    pub async fn new(
        config: &LightConfig,
        i2c_path: impl AsRef<Path>,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = LightController::new(&config.control, &gpio_path)
            .context("Failed to initialize light controller")?;

        Ok(Self {
            receiver,
            controller,
            sampler: LightSampler::new(&config.sample, sender, &i2c_path).await?,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        log::debug!("Starting light manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("All light manager tasks finished");
                    return Ok(());
                }
                Some(measurements) = self.receiver.recv() => {
                    log::info!("Light measurements: {measurements:?}");
                }
            }
        }
    }
}
