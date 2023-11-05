use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use super::Error;

#[derive(Debug, Parser)]
pub struct FanControlArgs {}

pub struct FanController {
    pin: OutputPin,
    on_duration: chrono::Duration,
    off_duration: chrono::Duration,
}

impl FanController {
    pub async fn start(
        cancel_token: CancellationToken,
        disable: bool,
        pin: u8,
        on_duration_secs: i64,
        off_duration_secs: i64,
    ) -> Result<(), Error> {
        if disable {
            log::info!("circulation fan controller is disabled by configuration");
            return Ok(());
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let mut pin = gpio.get(pin).map_err(Error::GetPinFailed)?.into_output();

        let on_duration = chrono::Duration::seconds(on_duration_secs);
        let off_duration = chrono::Duration::seconds(off_duration_secs);

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
