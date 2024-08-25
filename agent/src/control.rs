use anyhow::{bail, Context, Result};
use chrono::{NaiveTime, Utc};
use gpio_cdev::LineHandle;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub mod air_pump;
pub mod exhaust;
pub mod fan;
pub mod light;
pub mod water_level;

const GPIO_LOW: u8 = 0;
const GPIO_HIGH: u8 = 1;
const GPIO_CONSUMER: &str = "grow-agent";

pub struct CyclicController {
    handle: LineHandle,
    on_duration: Duration,
    off_duration: Duration,
}

impl CyclicController {
    fn new(handle: LineHandle, on_duration: Duration, off_duration: Duration) -> Self {
        Self {
            handle,
            on_duration,
            off_duration,
        }
    }

    async fn run(&mut self, cancel_token: CancellationToken, subject: &'static str) -> Result<()> {
        log::debug!("Starting {subject} controller");
        if self.off_duration.is_zero() {
            log::info!("The {subject} is always on");
            self.handle
                .set_value(GPIO_HIGH)
                .context("Failed to set value of control pin")?;
        }

        if self.on_duration.is_zero() {
            log::info!("The {subject} is always off");
            self.handle
                .set_value(GPIO_LOW)
                .context("Failed to set value of control pin")?;
        }

        log::debug!("Activating {subject}");
        self.handle
            .set_value(GPIO_HIGH)
            .context("Failed to set value of control pin")?;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    let value = self.handle
                        .get_value()
                        .context("Failed to get value of control pin")?;

                    if value == GPIO_HIGH {
                        log::debug!("Deactivating {subject}");
                        self.handle
                            .set_value(GPIO_LOW)
                            .context("Failed to set value of control pin")?;
                        timeout = self.on_duration;
                    } else {
                        log::debug!("Activating {subject}");
                        self.handle
                            .set_value(GPIO_HIGH)
                            .context("Failed to set value of control pin")?;
                        timeout = self.off_duration;

                    }
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("Stopping {subject} controller");
                    return Ok(());
                }
            }
        }
    }
}

pub struct TimeBasedController {
    handle: LineHandle,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
}

impl TimeBasedController {
    fn new(
        handle: LineHandle,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    ) -> Result<Self> {
        if activate_time == deactivate_time {
            bail!("Activate time and deactivate time cannot be equal");
        }

        Ok(Self {
            handle,
            activate_time,
            deactivate_time,
        })
    }

    async fn run(&mut self, cancel_token: CancellationToken, subject: &'static str) -> Result<()> {
        log::debug!("Starting {subject} controller");
        let mut timeout = Duration::from_secs(0);

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout)=> {
                    let now = Utc::now().time();
                    let until_on = match self.activate_time.signed_duration_since(now) {
                        dur if dur < chrono::Duration::zero() => dur
                            .checked_add(&chrono::Duration::days(1))
                            .expect("Duration until_on should not overflow"),
                        dur => dur,
                    };

                    let until_off = match self.deactivate_time.signed_duration_since(now) {
                        dur if dur < chrono::Duration::zero() => dur
                            .checked_add(&chrono::Duration::days(1))
                            .expect("Duration until_off should not overflow"),
                        dur => dur,
                    };

                    timeout = if until_on < until_off {
                        log::debug!("Deactivating {subject} now");
                        self.handle
                            .set_value(GPIO_LOW)
                            .context("Failed to set value of control pin")?;
                        log::info!(
                            "Activating {subject} in {:02}:{:02} h",
                            until_on.num_hours(),
                            until_on.num_minutes() % 60
                        );

                        until_on.to_std().context("Failed to convert chrono duration to std duration")?
                    } else {
                        log::debug!("Activating {subject} now");
                        self.handle
                            .set_value(GPIO_HIGH)
                            .context("Failed to set value of control pin")?;
                        log::info!(
                            "Deactivating {subject} in {:02}:{:02} h",
                            until_off.num_hours(),
                            until_off.num_minutes() % 60
                        );

                        until_off.to_std().context("Failed to convert chrono duration to std duration")?
                    };
                }
                _ = cancel_token.cancelled() => {
                    log::debug!("Stopping {subject} controller");
                    return Ok(());
                }
            }
        }
    }
}
