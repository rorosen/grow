use clap::Parser;
use tokio_util::sync::CancellationToken;

use super::{
    control::exhaust::{ExhaustControlArgs, ExhaustController},
    error::Error,
};

#[derive(Debug, Parser)]
pub struct ExhaustArgs {
    /// Whether to disable the exhaust fan controller
    #[arg(
        id = "exhaust_control_disable",
        long = "exhaust-control-disable",
        env = "GROW_AGENT_EXHAUST_CONTROL_DISABLE"
    )]
    pub disable: bool,

    #[command(flatten)]
    control: ExhaustControlArgs,

    /// The duration in seconds for which the exhaust fan should run (0 means always stopped)
    #[arg(
        id = "exhaust_control_on_duration_secs",
        long = "exhaust-control-on-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    pub on_duration_secs: i64,

    /// The duration in seconds for which the exhaust fan should be stopped (0 means always running)
    #[arg(
        id = "exhaust_control_off_duration_secs",
        long = "exhaust-control-off-duration-secs",
        env = "GROW_AGENT_EXHAUST_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    pub off_duration_secs: i64,
}

pub struct ExhaustManager {
    controller: ExhaustController,
    on_duration: chrono::Duration,
    off_duration: chrono::Duration,
}

impl ExhaustManager {
    pub async fn start(args: ExhaustArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        if args.disable {
            log::info!("exhaust fan controller is disabled by configuration");
            return Ok(());
        }

        let mut controller = ExhaustController::new(args.control.pin_slow)
            .await
            .map_err(Error::ControlError)?;

        let on_duration = chrono::Duration::seconds(args.on_duration_secs);
        let off_duration = chrono::Duration::seconds(args.off_duration_secs);

        if on_duration.is_zero() {
            log::info!("exhaust fan is always on");
            controller.deactivate_permanent();
            return Ok(());
        }

        if off_duration.is_zero() {
            log::info!("exhaust fan is always off");
            controller.activate_permanent();
            return Ok(());
        }

        Self {
            controller,
            on_duration,
            off_duration,
        }
        .run(cancel_token)
        .await
    }

    async fn run(mut self, cancel_token: CancellationToken) -> Result<(), Error> {
        log::debug!("starting exhaust fan management loop");
        self.controller.activate();
        let mut is_on = true;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    match is_on {
                        true => {
                            log::debug!("deactivating exhaust fan");
                            self.controller.deactivate();
                            is_on = false;
                            timeout = self.off_duration;
                        }
                        _ => {
                            log::debug!("activating exhaust fan");
                            self.controller.activate();
                            is_on = true;
                            timeout = self.on_duration;

                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping exhaust fan controller");
                    return Ok(());
                }
            }
        }
    }
}
