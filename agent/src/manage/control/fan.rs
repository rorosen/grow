use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use super::Error;

#[derive(Debug, Parser)]
pub struct FanControlArgs {
    /// Whether to disable the circulation fan controller
    #[arg(
        id = "fan_control_disable",
        long = "fan-control-disable",
        env = "GROW_AGENT_FAN_CONTROL_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to control the circulation fans
    #[arg(
        id = "fan_control_pin",
        long = "fan-control-pin",
        env = "GROW_AGENT_FAN_CONTROL_PIN",
        default_value_t = 23
    )]
    pub pin: u8,

    /// The duration in seconds for which the circulation fans should run (0 means always stopped)
    #[arg(
        id = "fan_control_on_duration_secs",
        long = "fan-control-on-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS",
        default_value_t = 1
    )]
    pub on_duration_secs: i64,

    /// The duration in seconds for which the circulation fans should be stopped (0 means always running)
    #[arg(
        id = "fan_control_off_duration_secs",
        long = "fan-control-off-duration-secs",
        env = "GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS",
        default_value_t = 0
    )]
    pub off_duration_secs: i64,
}

pub struct FanController {
    pin: OutputPin,
    on_duration: chrono::Duration,
    off_duration: chrono::Duration,
}

impl FanController {
    pub async fn start(args: FanControlArgs, cancel_token: CancellationToken) -> Result<(), Error> {
        if args.disable {
            log::info!("circulation fan controller is disabled by configuration");
            return Ok(());
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin = gpio
            .get(args.pin)
            .map_err(Error::GetPinFailed)?
            .into_output();

        let on_duration = chrono::Duration::seconds(args.on_duration_secs);
        let off_duration = chrono::Duration::seconds(args.off_duration_secs);

        if off_duration == chrono::Duration::zero() {
            log::info!("circulation fans are always on");
            pin.set_reset_on_drop(false);
            pin.set_high();
            return Ok(());
        }

        if on_duration == chrono::Duration::zero() {
            log::info!("circulation fans are always off");
            pin.set_reset_on_drop(false);
            pin.set_low();
            return Ok(());
        }

        Self {
            pin,
            on_duration,
            off_duration,
        }
        .run(cancel_token)
        .await
    }

    async fn run(mut self, cancel_token: CancellationToken) -> Result<(), Error> {
        log::debug!("starting circulation fan controller");
        self.pin.set_high();
        let mut is_on = true;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    match is_on {
                        true => {
                            log::debug!("deactivating circulation fans");
                            self.pin.set_low();
                            is_on = false;
                            timeout = self.off_duration;
                        }
                        _ => {
                            log::debug!("activating circulation fans");
                            is_on = true;
                            timeout = self.on_duration;

                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping circulation fan controller");
                    return Ok(());
                }
            }
        }
    }
}
