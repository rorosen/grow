use super::{
    control::light::{LightControlArgs, LightController},
    sample::light::{LightSampleArgs, LightSampler},
};
use crate::error::AppError;
use clap::Parser;
use common::LightMeasurement;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

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

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting light manager");

        tokio::pin! {
            let control_task = tokio::spawn(self.controller.run(cancel_token.clone()));
            let sample_task = tokio::spawn(self.sampler.run(cancel_token));
        };

        loop {
            tokio::select! {
                res = &mut control_task, if control_task.is_finished() => {
                    match res {
                        Ok(Ok(_)) => log::info!("light controller task finished"),
                        Ok(Err(err)) => {
                            log::error!("light controller aborted with an error: {err}");
                            return Err(err);
                        }
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"light controller",err,});
                        }
                    }
                }
                res = &mut sample_task, if sample_task.is_finished() => {
                    match res {
                        Ok(_) => log::info!("light sample task finished"),
                        Err(err) => {
                            return Err(AppError::TaskPanicked{name:"light sample",err,});
                        }
                    }
                }
                Some((id, measurement)) = self.receiver.recv() => {
                    log::info!("received {id} light measurement: {measurement:?}");
                }
            }
        }
    }
}
