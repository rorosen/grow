use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::{NaiveTime, Utc};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

pub mod air;
pub mod air_pump;
pub mod fan;
pub mod light;
pub mod water_level;

const GPIO_DEACTIVATE: u8 = 0;
const GPIO_ACTIVATE: u8 = 1;
const GPIO_CONSUMER: &str = "grow-agent";

#[async_trait]
trait Control {
    async fn run(
        &mut self,
        cancel_token: CancellationToken,
        identifier: &'static str,
    ) -> Result<()>;
}

struct CyclicController {
    handle: LineHandle,
    on_duration: Duration,
    off_duration: Duration,
}

impl CyclicController {
    fn new(
        gpio_path: impl AsRef<Path>,
        pin: u32,
        on_duration: Duration,
        off_duration: Duration,
    ) -> Result<Self> {
        let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
        let handle = chip
            .get_line(pin)
            .with_context(|| format!("Failed to get handle to GPIO line {pin}"))?
            .request(LineRequestFlags::OUTPUT, GPIO_DEACTIVATE, GPIO_CONSUMER)
            .with_context(|| format!("Failed to get access to GPIO {pin}"))?;

        Ok(Self {
            handle,
            on_duration,
            off_duration,
        })
    }
}

#[async_trait]
impl Control for CyclicController {
    async fn run(
        &mut self,
        cancel_token: CancellationToken,
        identifier: &'static str,
    ) -> Result<()> {
        if self.off_duration.is_zero() {
            log::info!("{identifier}: Activating control pin permanently");
            self.handle
                .set_value(GPIO_ACTIVATE)
                .context("Failed to set value of control pin")?;

            cancel_token.cancelled().await;
            return Ok(());
        }

        if self.on_duration.is_zero() {
            log::info!("{identifier}: Deactivating control pin permanently");
            self.handle
                .set_value(GPIO_DEACTIVATE)
                .context("Failed to set value of control pin")?;

            cancel_token.cancelled().await;
            return Ok(());
        }

        log::debug!("{identifier}: Activating control pin");
        self.handle
            .set_value(GPIO_ACTIVATE)
            .context("Failed to set value of control pin")?;
        let mut timeout = self.on_duration;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    let value = self.handle
                        .get_value()
                        .context("Failed to get value of control pin")?;

                    if value == GPIO_ACTIVATE {
                        log::debug!("{identifier}: Deactivating control pin");
                        self.handle
                            .set_value(GPIO_DEACTIVATE)
                            .context("Failed to set value of control pin")?;
                        timeout = self.on_duration;
                    } else {
                        log::debug!("{identifier}: Activating control pin");
                        self.handle
                            .set_value(GPIO_ACTIVATE)
                            .context("Failed to set value of control pin")?;
                        timeout = self.off_duration;
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}

struct TimeBasedController {
    handle: LineHandle,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
}

impl TimeBasedController {
    fn new(
        gpio_path: impl AsRef<Path>,
        pin: u32,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    ) -> Result<Self> {
        if activate_time == deactivate_time {
            bail!("Activate time and deactivate time cannot be equal");
        }

        let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
        let handle = chip
            .get_line(pin)
            .with_context(|| format!("Failed to get handle to GPIO line {pin}"))?
            .request(LineRequestFlags::OUTPUT, GPIO_DEACTIVATE, GPIO_CONSUMER)
            .with_context(|| format!("Failed to get access to GPIO {pin}"))?;

        Ok(Self {
            handle,
            activate_time,
            deactivate_time,
        })
    }
}

#[async_trait]
impl Control for TimeBasedController {
    async fn run(
        &mut self,
        cancel_token: CancellationToken,
        identifier: &'static str,
    ) -> Result<()> {
        const ACTION_ACTIVATE: &str = "Activating";
        const ACTION_DEACTIVATE: &str = "Deactivating";

        let mut timeout = Duration::from_secs(0);
        let set_pin = |value: u8, dur: chrono::Duration| -> Result<Duration> {
            let actions = if value == GPIO_ACTIVATE {
                (ACTION_ACTIVATE, ACTION_DEACTIVATE)
            } else {
                (ACTION_DEACTIVATE, ACTION_ACTIVATE)
            };

            log::debug!("{identifier}: {} control pin", actions.0);
            self.handle
                .set_value(value)
                .context("Failed to set value of control pin")?;

            log::debug!(
                "{identifier}: {} control pin in {:02}:{:02}:{:02}h",
                actions.1,
                dur.num_hours(),
                dur.num_minutes() % 60,
                dur.num_seconds() % 60
            );

            let ret = dur
                .to_std()
                .context("Failed to convert chrono duration to std duration")?;

            Ok(ret)
        };

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout)=> {
                    let now = Utc::now().time();
                    let until_on = match self.activate_time.signed_duration_since(now) {
                        dur if dur < chrono::Duration::zero() => dur
                            .checked_add(&chrono::Duration::days(1))
                            .context("Failed to add day to until on")?,
                        dur => dur,
                    };

                    let until_off = match self.deactivate_time.signed_duration_since(now) {
                        dur if dur < chrono::Duration::zero() => dur
                            .checked_add(&chrono::Duration::days(1))
                            .context("Failed to add day to until off")?,
                        dur => dur,
                    };

                    timeout = if until_on < until_off {
                        set_pin(GPIO_DEACTIVATE, until_on)?
                    } else {
                        set_pin(GPIO_ACTIVATE, until_off)?
                    };
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}
