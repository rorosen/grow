use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::Error;

#[derive(Debug, Parser)]
pub struct ExhaustControlArgs {
    /// Whether to disable the exhaust fan controller
    #[arg(
        id = "exhaust_control_disable",
        long = "exhaust-control-disable",
        env = "GROW_AGENT_EXHAUST_CONTROL_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to enable the exhaust fan in slow mode
    #[arg(
        id = "exhaust_control_pin_slow",
        long = "exhaust-control-pin-slow",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_SLOW",
        default_value_t = 25
    )]
    pub pin_slow: u8,

    /// The gpio pin used to enable the exhaust fan in fast mode (not implemented so far)
    #[arg(
        id = "exhaust_control_pin_fast",
        long = "exhaust-control-pin-fast",
        env = "GROW_AGENT_EXHAUST_CONTROL_PIN_FAST",
        default_value_t = 26
    )]
    pub pin_fast: u8,

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

pub struct ExhaustController {
    pin: OutputPin,
    cancel_token: CancellationToken,
    on_duration: chrono::Duration,
    off_duration: chrono::Duration,
}

impl ExhaustController {
    pub fn start(
        args: ExhaustControlArgs,
        cancel_token: CancellationToken,
        finish: mpsc::Sender<()>,
    ) -> Result<(), Error> {
        if args.disable {
            log::info!("exhaust fan controller is disabled by configuration");
            return Ok(());
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin = gpio
            .get(args.pin_slow)
            .map_err(Error::GetPinFailed)?
            .into_output_low();

        let on_duration = chrono::Duration::seconds(args.on_duration_secs);
        let off_duration = chrono::Duration::seconds(args.off_duration_secs);

        if off_duration == chrono::Duration::zero() {
            log::info!("exhaust fan is always on");
            pin.set_reset_on_drop(false);
            pin.set_high();
            return Ok(());
        }

        if on_duration == chrono::Duration::zero() {
            log::info!("exhaust fan is always off");
            pin.set_reset_on_drop(false);
            pin.set_low();
            return Ok(());
        }

        tokio::spawn(
            Self {
                pin,
                cancel_token,
                on_duration,
                off_duration,
            }
            .run(finish),
        );

        Ok(())
    }

    pub async fn run(mut self, _finish: mpsc::Sender<()>) {
        log::debug!("starting exhaust fan controller");
        self.pin.set_high();
        let mut is_on = true;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    match is_on {
                        true => {
                            log::debug!("deactivating exhaust fan");
                            self.pin.set_low();
                            is_on = false;
                            timeout = self.off_duration;
                        }
                        _ => {
                            log::debug!("activating exhaust fan");
                            self.pin.set_high();
                            is_on = true;
                            timeout = self.on_duration;

                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    log::debug!("stopping exhaust fan controller");
                    return;
                }
            }
        }
    }
}
