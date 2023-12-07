use super::{
    control::exhaust::{ExhaustControlArgs, ExhaustController},
    sample::air::{AirSampleArgs, AirSampler},
};

use crate::error::AppError;
use clap::Parser;
use common::AirMeasurement;
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
    receiver: mpsc::Receiver<(&'static str, AirMeasurement)>,
    controller: ExhaustController,
    sampler: AirSampler,
}

impl AirManager {
    pub fn new(args: &AirArgs) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::channel(8);

        Ok(Self {
            receiver,
            controller: ExhaustController::new(&args.control)?,
            sampler: AirSampler::new(&args.sample, sender),
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
                Some((id, measurement)) = self.receiver.recv() => {
                    log::info!("received {id} air measurement: {measurement:?}");
                }
            }
        }
    }
}
