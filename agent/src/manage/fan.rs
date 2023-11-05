use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{
    control::fan::{FanControlArgs, FanController},
    error::Error,
};

#[derive(Debug, Parser)]
pub struct FanArgs {
    #[command(flatten)]
    control: FanControlArgs,
}

pub struct FanManager {
    args: FanArgs,
}

impl FanManager {
    pub async fn start(args: FanArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        Self { args }.run(cancel_token).await
    }

    async fn run(self, cancel_token: CancellationToken) -> Result<(), Error> {
        FanController::start(self.args.control, cancel_token)
            .await
            .map_err(Error::ControlError)
    }
}
