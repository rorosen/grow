use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio::sync::mpsc;
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

    /// The gpio pin used to disable the left circulation fan
    #[arg(
        id = "fan_control_pin_left",
        long = "fan-control-pin-left",
        env = "GROW_AGENT_FAN_CONTROL_PIN_LEFT",
        default_value_t = 23
    )]
    pub pin_left: u8,

    /// The gpio pin used to disable the right circulation fan
    #[arg(
        id = "fan_control_pin_right",
        long = "fan-control-pin-right",
        env = "GROW_AGENT_FAN_CONTROL_PIN_RIGHT",
        default_value_t = 24
    )]
    pub pin_right: u8,

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
    pin_left: OutputPin,
    pin_right: OutputPin,
    cancel_token: CancellationToken,
    on_duration: chrono::Duration,
    off_duration: chrono::Duration,
}

impl FanController {
    pub fn start(
        args: FanControlArgs,
        cancel_token: CancellationToken,
        finish: mpsc::Sender<()>,
    ) -> Result<(), Error> {
        if args.disable {
            log::info!("circulation fan controller is disabled by configuration");
            return Ok(());
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin_left = gpio
            .get(args.pin_left)
            .map_err(Error::GetPinFailed)?
            .into_output();

        let mut pin_right = gpio
            .get(args.pin_right)
            .map_err(Error::GetPinFailed)?
            .into_output();

        let on_duration = chrono::Duration::seconds(args.on_duration_secs);
        let off_duration = chrono::Duration::seconds(args.off_duration_secs);

        if off_duration == chrono::Duration::zero() {
            log::info!("circulation fans are always on");
            pin_left.set_reset_on_drop(false);
            pin_right.set_reset_on_drop(false);
            pin_left.set_high();
            pin_right.set_high();
            return Ok(());
        }

        if on_duration == chrono::Duration::zero() {
            log::info!("circulation fans are always off");
            pin_left.set_reset_on_drop(false);
            pin_right.set_reset_on_drop(false);
            pin_left.set_low();
            pin_right.set_low();
            return Ok(());
        }

        tokio::spawn(
            Self {
                pin_left,
                pin_right,
                cancel_token,
                on_duration,
                off_duration,
            }
            .run(finish),
        );

        Ok(())
    }

    pub async fn run(mut self, _finish: mpsc::Sender<()>) {
        log::debug!("starting circulation fan controller");
        self.pin_left.set_high();
        self.pin_right.set_high();
        let mut is_on = true;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    match is_on {
                        true => {
                            log::debug!("deactivating circulation fan");
                            self.pin_left.set_low();
                            self.pin_right.set_low();
                            is_on = false;
                            timeout = self.off_duration;
                        }
                        _ => {
                            log::debug!("activating circulation fan");
                            self.pin_left.set_high();
                            self.pin_right.set_high();
                            is_on = true;
                            timeout = self.on_duration;

                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    log::debug!("stopping circulation fan controller");
                    return;
                }
            }
        }
    }
}
