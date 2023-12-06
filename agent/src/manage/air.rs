use super::{control::exhaust::ExhaustController, sample::air::AirSampler, ExhaustControlArgs};
use crate::{error::AppError, manage::control::control_cyclic};
use clap::{Parser, ValueEnum};
use common::AirMeasurement;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct AirArgs {
    #[command(flatten)]
    control: ExhaustControlArgs,

    #[command(flatten)]
    sample: AirSampleArgs,
}

pub struct AirManager {
    controller: ExhaustController,
    left_address: u8,
    right_address: u8,
}

impl AirManager {
    pub async fn new(args: &AirArgs) -> Result<Self, AppError> {
        Ok(Self {
            controller: ExhaustController::new(&args.control)?,
            left_address: args.sample.left_sensor_address,
            right_address: args.sample.right_sensor_address,
        })
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<(), AppError> {
        log::debug!("starting air manager");

        let controller_handle = tokio::spawn(self.controller.run(cancel_token.clone()));
        let mut left_sampler = AirSampler::new(self.left_address).await.ok();
        let mut right_sampler = AirSampler::new(self.right_address).await.ok();

        loop {
            if left_sampler.is_none() {
                left_sampler = AirSampler::new(self.left_address).await.ok();
            }

            if right_sampler.is_none() {
                right_sampler = AirSampler::new(self.right_address).await.ok();
            }
        }

        Ok(())
    }
}
