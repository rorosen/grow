use super::{
    control::exhaust::{ExhaustControlArgs, ExhaustController},
    sample::air::{AirSample, AirSampleArgs, AirSampler},
};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Debug, Parser)]
pub struct AirArgs {
    #[command(flatten)]
    control: ExhaustControlArgs,

    #[command(flatten)]
    sample: AirSampleArgs,
}

pub struct AirManager {
    receiver: mpsc::Receiver<AirSample>,
    controller: ExhaustController,
    sampler: AirSampler,
}

impl AirManager {
    pub async fn new(args: &AirArgs) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller = ExhaustController::new(&args.control)
            .context("failed to initialize exhaust fan controller")?;
        let sampler = AirSampler::new(&args.sample, sender).await?;

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
