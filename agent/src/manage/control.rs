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

pub struct CyclicController {
    pin: OutputPin,
    on_duration: Duration,
    off_duration: Duration,
}

impl CyclicController {
    fn new(pin: OutputPin, on_duration: Duration, off_duration: Duration) -> Self {
        Self {
            pin,
            on_duration,
            off_duration,
        }
    }

    async fn run(&mut self, cancel_token: CancellationToken, subject: &'static str) {
        log::debug!("starting {subject} controller");
        if self.off_duration.is_zero() {
            log::info!("{subject} is always on");
            self.pin.set_reset_on_drop(false);
            self.pin.set_high();
            return;
        }

        if self.on_duration.is_zero() {
            log::info!("{subject} is always off");
            self.pin.set_reset_on_drop(false);
            self.pin.set_low();
            return;
        }

        self.pin.set_high();
        let timeout = |is_on: bool| {
            if is_on {
                self.on_duration
            } else {
                self.off_duration
            }
        };

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout(self.pin.is_set_high())) => {
                    match self.pin.is_set_high() {
                        true => {
                            log::debug!("deactivating {subject}");
                            self.pin.set_low();
                        }
                        _ => {
                            log::debug!("activating {subject}");
                            self.pin.set_high();
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping {subject} controller");
                    return;
                }
            }
        }
    }
}

pub struct TimeBasedController {
    pin: OutputPin,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
}

impl TimeBasedController {
    fn new(pin: OutputPin, activate_time: NaiveTime, deactivate_time: NaiveTime) -> Result<Self> {
        if activate_time == deactivate_time {
            bail!("activate time and deactivate time cannot be equal");
        }

        Ok(Self {
            pin,
            activate_time,
            deactivate_time,
        })
    }

    async fn run(&mut self, cancel_token: CancellationToken, subject: &'static str) {
        log::debug!("starting {subject} controller");
        let mut timeout = chrono::Duration::zero();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
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

                    timeout = if until_on < until_off {
                        log::debug!("deactivating {subject} now");
                        self.pin.set_low();
                        log::info!(
                            "activating {subject} in {:02}:{:02} h",
                            until_on.num_hours(),
                            until_on.num_minutes() % 60
                        );

                        until_on
                    } else {
                        log::debug!("activating {subject} now");
                        self.pin.set_high();
                        log::info!(
                            "deactivating {subject} in {:02}:{:02} h",
                            until_off.num_hours(),
                            until_off.num_minutes() % 60
                        );

                        until_off
                    };
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("stopping {subject} controller");
                    return;
                }
            }
        }
    }
}
