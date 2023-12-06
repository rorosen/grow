use super::{
    control::pump::{PumpControlArgs, PumpController},
    sample::water_level::{WaterLevelSampleArgs, WaterLevelSampler},
};
use crate::error::AppError;
use clap::Parser;
use common::WaterLevelMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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
    pub async fn new(args: WaterArgs) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::channel(8);

        Ok(Self {
            receiver,
            controller: PumpController::new(&args.control)?,
            sampler: WaterLevelSampler::new(&args.sample, sender),
        })
    }

    async fn run(mut self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting water manager");

        tokio::pin! {
            let control_task = tokio::spawn(self.controller.run(cancel_token.clone()));
            let sample_task = tokio::spawn(self.sampler.run(cancel_token));
        };

        loop {
            tokio::select! {
                res = &mut control_task, if control_task.is_finished() => {
                    match res {
                        Ok(_) => log::info!("pump controller task finished"),
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"pump controller",err,});
                        }
                    }
                }
                res = &mut sample_task, if sample_task.is_finished() => {
                    match res {
                        Ok(_) => log::info!("water level sample task finished"),
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"water level sample",err,});
                        }
                    }
                }
                Some(measurement) = self.receiver.recv() => {
                    log::info!("received water level measurement: {measurement:?}");
                }
            }
        }
    }
}
