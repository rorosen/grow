use super::{
    control::pump::{PumpControlArgs, PumpController},
    sample::water_level::{WaterLevelSampleArgs, WaterLevelSampler},
};
use crate::error::AppError;
use clap::Parser;
use common::WaterLevelMeasurement;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Debug, Parser)]
pub struct WaterArgs {
    #[command(flatten)]
    control: PumpControlArgs,

    #[command(flatten)]
    sample: WaterLevelSampleArgs,
}

pub struct WaterManager {
    receiver: mpsc::Receiver<(&'static str, WaterLevelMeasurement)>,
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl WaterManager {
    pub fn new(args: &WaterArgs) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::channel(8);

        Ok(Self {
            receiver,
            controller: PumpController::new(&args.control)?,
            sampler: WaterLevelSampler::new(&args.sample, sender),
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
                Some((id, measurement)) = self.receiver.recv() => {
                    log::info!("received {id} water level measurement: {measurement:?}");
                }
            }
        }
    }
}
