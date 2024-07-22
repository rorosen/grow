use anyhow::{bail, Result};
use chrono::{NaiveTime, Utc};
use rppal::gpio::OutputPin;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub mod air_pump;
pub mod exhaust;
pub mod fan;
pub mod light;
pub mod pump;

pub async fn control_cyclic(
    pin: &mut OutputPin,
    on_duration: Duration,
    off_duration: Duration,
    cancel_token: CancellationToken,
    controller_name: &str,
) {
    if off_duration.is_zero() {
        log::info!("{controller_name} is always on");
        pin.set_reset_on_drop(false);
        pin.set_high();
        return;
    }

    if on_duration.is_zero() {
        log::info!("{controller_name} is always off");
        pin.set_reset_on_drop(false);
        pin.set_low();
        return;
    }

    pin.set_high();
    let timeout = |is_on: bool| if is_on { on_duration } else { off_duration };

    loop {
        tokio::select! {
            _ = tokio::time::sleep(timeout(pin.is_set_high())) => {
                match pin.is_set_high() {
                    true => {
                        log::debug!("deactivating {controller_name}");
                        pin.set_low();
                    }
                    _ => {
                        log::debug!("activating {controller_name}");
                        pin.set_high();
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                log::debug!("stopping {controller_name} manager");
                return;
            }
        }
    }
}

pub async fn control_time_based(
    pin: &mut OutputPin,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
    cancel_token: CancellationToken,
    controller_name: &str,
) -> Result<()> {
    if activate_time == deactivate_time {
        bail!("");
        // return Err(Error::InvalidControllerArgs(
        //     controller_name.into(),
        //     "activate time and deactivate time cannot be equal".into(),
        // ));
    }

    log::debug!("starting {controller_name} manager");
    let mut timeout = chrono::Duration::zero();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                let now = Utc::now().time();

                let until_on = match activate_time.signed_duration_since(now) {
                    dur if dur < chrono::Duration::zero() => {
                        // should never be none
                        dur.checked_add(&chrono::Duration::days(1)).unwrap()
                    }
                    dur => dur,
                };

                let until_off = match deactivate_time.signed_duration_since(now) {
                    dur if dur < chrono::Duration::zero() => {
                        dur.checked_add(&chrono::Duration::days(1)).unwrap()
                    }
                    dur => dur,
                };

                timeout = if until_on < until_off {
                    log::debug!("deactivating {controller_name} now");
                    pin.set_low();
                    log::info!(
                        "activating {controller_name} in {:02}:{:02} h",
                        until_on.num_hours(),
                        until_on.num_minutes() % 60
                    );

                    until_on
                } else {
                    log::debug!("activating {controller_name} now");
                    pin.set_high();
                    log::info!(
                        "deactivating {controller_name} in {:02}:{:02} h",
                        until_off.num_hours(),
                        until_off.num_minutes() % 60
                    );

                    until_off
                };
            }
            _ = cancel_token.cancelled() => {
                log::debug!("stopping {controller_name} control");
                return Ok(());
            }
        }
    }
}
