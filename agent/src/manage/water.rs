use super::{
    control::pump::{PumpControlArgs, PumpController},
    sample::water_level::{WaterLevelSampleArgs, WaterLevelSampler},
};
use anyhow::{Context, Result};
use clap::Parser;
use grow_measure::WaterLevelMeasurement;
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
    receiver: mpsc::Receiver<WaterLevelMeasurement>,
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl WaterManager {
    pub fn new(args: &WaterArgs) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(8);
        let controller =
            PumpController::new(&args.control).context("failed to initialize pump controller")?;

        Ok(Self {
            receiver,
            controller,
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
                Some(measurement) = self.receiver.recv() => {
                    log::info!("water level measurement: {measurement:?}");
                }
            }
        }
    }
}
