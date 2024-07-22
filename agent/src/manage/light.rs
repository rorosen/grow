use super::{
    control::light::{LightControlArgs, LightController},
    sample::light::{LightSample, LightSampleArgs, LightSampler},
};
use anyhow::{Context, Result};
use clap::Parser;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Debug, Parser)]
pub struct LightArgs {
    #[command(flatten)]
    control: LightControlArgs,

    #[command(flatten)]
    sample: LightSampleArgs,
}

pub struct LightManager {
    receiver: mpsc::Receiver<LightSample>,
    controller: LightController,
    sampler: LightSampler,
}

impl LightManager {
    pub async fn new(args: &LightArgs) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller =
            LightController::new(&args.control).context("failed to initialize light controller")?;

        Ok(Self {
            receiver,
            controller,
            sampler: LightSampler::new(&args.sample, sender).await?,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) {
        log::debug!("starting light manager");

        let tracker = TaskTracker::new();
        tracker.spawn(self.controller.run(cancel_token.clone()));
        tracker.spawn(self.sampler.run(cancel_token));
        tracker.close();

        loop {
            tokio::select! {
                _ = tracker.wait() => {
                    log::debug!("all light manager tasks finished");
                    return;
                }
                Some(LightSample{left, right, ..}) = self.receiver.recv() => {
                    log::info!("left light measurement: {left:?}");
                    log::info!("right light measurement: {right:?}");
                }
            }
        }
    }
}
