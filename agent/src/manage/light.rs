use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{
    control::light::{LightControlArgs, LightController},
    error::Error,
};

#[derive(Debug, Parser)]
pub struct LightArgs {
    #[command(flatten)]
    control: LightControlArgs,
}

pub struct LightManager {
    args: LightArgs,
}

impl LightManager {
    pub async fn start(args: LightArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        Self { args }.run(cancel_token).await
    }

    async fn run(self, cancel_token: CancellationToken) -> Result<(), Error> {
        LightController::start(self.args.control, cancel_token)
            .await
            .map_err(Error::ControlError)
    }
}
