use super::{
    control::exhaust::ExhaustController,
    sample::air::{AirSampleArgs, AirSampler},
    ExhaustControlArgs,
};

use crate::error::AppError;
use clap::Parser;
use common::AirMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct AirArgs {
    #[command(flatten)]
    control: ExhaustControlArgs,

    #[command(flatten)]
    sample: AirSampleArgs,
}

pub struct AirManager {
    receiver: mpsc::Receiver<AirMeasurement>,
    controller: ExhaustController,
    sampler: AirSampler,
}

impl AirManager {
    pub async fn new(args: &AirArgs) -> Result<Self, AppError> {
        let (sender, receiver) = mpsc::channel(8);

        Ok(Self {
            receiver,
            controller: ExhaustController::new(&args.control)?,
            sampler: AirSampler::new(&args.sample, sender),
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting air manager");

        tokio::pin! {
            let control_task = tokio::spawn(self.controller.run(cancel_token.clone()));
            let sample_task = tokio::spawn(self.sampler.run(cancel_token));
        };

        loop {
            tokio::select! {
                res = &mut control_task, if control_task.is_finished() => {
                    match res {
                        Ok(Ok(_)) => log::info!("exhaust controller task finished"),
                        Ok(Err(err)) => {
                            log::error!("exhaust controller aborted with an error: {err}");
                            return Err(err);
                        }
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"exhaust controller",err,});
                        }
                    }
                }
                res = &mut sample_task, if sample_task.is_finished() => {
                    match res {
                        Ok(_) => log::info!("air sample task finished"),
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"air sample",err,});
                        }
                    }
                }
                Some(measurement) = self.receiver.recv() => {
                    log::info!("received air measurement: {measurement:?}");
                }
            }
        }
    }
}
