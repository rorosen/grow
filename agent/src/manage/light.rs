use super::{
    control::light::{LightControlArgs, LightController},
    sample::light::{LightSampleArgs, LightSampler},
};
use crate::error::AppError;
use clap::Parser;
use common::LightMeasurement;
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
    receiver: mpsc::Receiver<(&'static str, LightMeasurement)>,
    controller: LightController,
    sampler: LightSampler,
}

impl LightManager {
    pub fn new(args: &LightArgs) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::channel(8);

        Ok(Self {
            receiver,
            controller: LightController::new(&args.control)?,
            sampler: LightSampler::new(&args.sample, sender),
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
                Some((id, measurement)) = self.receiver.recv() => {
                    log::info!("received {id} light measurement: {measurement:?}");
                }
            }
        }
    }
}
