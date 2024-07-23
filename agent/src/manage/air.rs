use crate::config::air::AirConfig;

use super::{
    control::exhaust::ExhaustController,
    sample::air::{AirSample, AirSampler},
};

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct AirManager {
    receiver: mpsc::Receiver<AirSample>,
    controller: ExhaustController,
    sampler: AirSampler,
}

impl AirManager {
    pub async fn new(config: &AirConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = ExhaustController::new(&config.control)
            .context("failed to initialize exhaust fan controller")?;
        let sampler = AirSampler::new(&config.sample, sender).await?;

        Ok(Self {
            receiver,
            controller,
            sampler,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("starting air manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("all air manager tasks finished");
                    return;
                }
                Some(AirSample{left, right, ..}) = self.receiver.recv() => {
                    log::info!("left air measurement: {left:?}");
                    log::info!("right air measurement: {right:?}");
                }
            }
        }
    }
}
