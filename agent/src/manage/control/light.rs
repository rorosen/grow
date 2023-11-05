use chrono::{NaiveTime, Utc};
use rppal::gpio::{Gpio, OutputPin};
use tokio_util::sync::CancellationToken;

use super::error::Error;

pub struct LightController {
    pin: OutputPin,
    cancel_token: CancellationToken,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
}

impl LightController {
    pub async fn start(
        cancel_token: CancellationToken,
        disable: bool,
        pin: u8,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    ) -> Result<(), Error> {
        if disable {
            log::info!("light controller is disabled by configuration");
            return Ok(());
        }

        if activate_time == deactivate_time {
            return Err(Error::InvalidArgs(
                "light".into(),
                "activate time and deactivate time cannot be equal".into(),
            ));
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let pin = gpio.get(pin).map_err(Error::GetPinFailed)?.into_output();

        Self {
            pin,
            cancel_token,
            activate_time,
            deactivate_time,
        }
        .run()
        .await
    }

    pub async fn run(mut self) -> Result<(), Error> {
        log::debug!("starting light controller");
        let mut timeout = chrono::Duration::zero();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    timeout = self.control();
                }
                _ = self.cancel_token.cancelled() => {
                    log::debug!("stopping light controller");
                    return Ok(());
                }
            }
        }
    }

    fn control(&mut self) -> chrono::Duration {
        let now = Utc::now().time();

        let until_on = match self.activate_time.signed_duration_since(now) {
            dur if dur < chrono::Duration::zero() => {
                // should never be none
                dur.checked_add(&chrono::Duration::days(1)).unwrap()
            }
            dur => dur,
        };

        let until_off = match self.deactivate_time.signed_duration_since(now) {
            dur if dur < chrono::Duration::zero() => {
                dur.checked_add(&chrono::Duration::days(1)).unwrap()
            }
            dur => dur,
        };

        if until_on < until_off {
            log::debug!("deactivating light now");
            self.pin.set_low();

            log::info!(
                "activating light in {:02}:{:02} h",
                until_on.num_hours(),
                until_on.num_minutes() % 60
            );
            until_on
        } else {
            log::debug!("activating light now");
            self.pin.set_high();

            log::info!(
                "deactivating light in {:02}:{:02} h",
                until_off.num_hours(),
                until_off.num_minutes() % 60
            );
            until_off
        }
    }
}
