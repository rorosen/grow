use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::{NaiveTime, Utc};
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use std::{path::Path, time::Duration};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

pub trait ThresholdControl
where
    Self: Sized,
{
    type Threshold;

    fn threshold(activate_condition: &str, deactivate_condition: &str) -> Result<Self::Threshold>;
    fn desired_state(values: &[Self], threshold: &Self::Threshold) -> Result<Option<GpioState>>;
}


const GPIO_CONSUMER: &str = "grow-agent";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpioState {
    Activated,
    Deactivated,
}

impl GpioState {
    fn as_str(&self) -> &str {
        match self {
            Self::Activated => "Activating",
            Self::Deactivated => "Deactivating",
        }
    }

    fn from_value(value: u8) -> Self {
        if value == 1 {
            return Self::Activated;
        }

        Self::Deactivated
    }

    fn to_value(&self) -> u8 {
        match self {
            Self::Activated => 1,
            Self::Deactivated => 0,
        }
    }

    fn toggle(&self) -> Self {
        match self {
            Self::Activated => Self::Deactivated,
            Self::Deactivated => Self::Activated,
        }
    }
}

#[async_trait]
pub trait Control {
    async fn run(&mut self, cancel_token: CancellationToken) -> Result<()>;
}

// TODO: deactivate pin before terminating
pub struct Controller {
    inner: Option<Box<dyn Control + Send>>,
}

impl Controller {
    pub fn new_disabled() -> Self {
        Self { inner: None }
    }

    pub fn new_cyclic(
        gpio_path: impl AsRef<Path>,
        pin: u32,
        on_duration: Duration,
        off_duration: Duration,
    ) -> Result<Self> {
        let controller = CyclicController::new(
            gpio_path,
            pin,
            on_duration,
            off_duration,
        )
        .context("Failed to initilaize cyclic controller")?;

        Ok(Self {
            inner: Some(Box::new(controller)),
        })
    }

    pub fn new_time_based(
        gpio_path: impl AsRef<Path>,
        pin: u32,
        activate_time: NaiveTime,
        deactivate_time: NaiveTime,
    ) -> Result<Self> {
        let controller = TimeBasedController::new(gpio_path, pin, activate_time, deactivate_time)
            .context("Failed to initialize time based controller")?;

        Ok(Self {
            inner: Some(Box::new(controller)),
        })
    }

    pub fn new_threshold<M>(
        activate_condition: &str,
        deactivate_condition: &str,
        gpio_path: impl AsRef<Path>,
        pin: u32,
        receiver: broadcast::Receiver<Vec<M>>,
    ) -> Result<Self>
    where
        M: ThresholdControl + Send + Clone + 'static,
        M::Threshold: Send,
    {
        let controller = ThresholdController::new(
            activate_condition,
            deactivate_condition,
            gpio_path,
            pin,
            receiver,
        )
        .context("Failed to initialize threshold controller")?;

        Ok(Self {
            inner: Some(Box::new(controller)),
        })
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        if let Some(mut controller) = self.inner {
            controller
                .run(cancel_token)
                .await
                .context("Failed to run controller")?;
        } else {
            info!("Controller is disabled");
        }

        Ok(())
    }
}

pub struct CyclicController {
    handle: LineHandle,
    on_duration: Duration,
    off_duration: Duration,
}

impl CyclicController {
    pub fn new(
        gpio_path: impl AsRef<Path>,
        pin: u32,
        on_duration: Duration,
        off_duration: Duration,
    ) -> Result<Self> {
        let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
        let handle = chip
            .get_line(pin)
            .with_context(|| format!("Failed to get handle to GPIO line {pin}"))?
            .request(
                LineRequestFlags::OUTPUT,
                GpioState::Deactivated.to_value(),
                GPIO_CONSUMER,
            )
            .with_context(|| format!("Failed to get access to GPIO {pin}"))?;

        Ok(Self {
            handle,
            on_duration,
            off_duration,
        })
    }

    fn next_timout(&self, state: GpioState) -> Duration {
        match state {
            GpioState::Activated => self.on_duration,
            GpioState::Deactivated => self.off_duration,
        }
    }
}

#[async_trait]
impl Control for CyclicController {
    async fn run(&mut self, cancel_token: CancellationToken) -> Result<()> {
        let make_suffix = |cond: bool| if cond { " permanently" } else { "" };
        let (initial_state, is_initial_state_permanent) = if self.off_duration.is_zero() {
            (GpioState::Activated, true)
        } else if self.on_duration.is_zero() {
            (GpioState::Deactivated, true)
        } else {
            (GpioState::Activated, false)
        };

        info!(
            "{} control pin{}",
            initial_state.as_str(),
            make_suffix(is_initial_state_permanent)
        );
        self.handle
            .set_value(initial_state.to_value())
            .context("Failed to set value of control pin")?;

        if is_initial_state_permanent {
            cancel_token.cancelled().await;
            return Ok(());
        }

        let mut timeout = self.next_timout(initial_state);
        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout) => {
                    let value = self.handle
                        .get_value()
                        .context("Failed to get value of control pin")?;

                    let state = GpioState::from_value(value).toggle();
                    debug!("{} control pin", state.as_str());
                    self.handle
                        .set_value(state.to_value())
                        .context("Failed to set value of control pin")?;
                    timeout = self.next_timout(state);
                }
                _ = cancel_token.cancelled() => {
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
    pub fn new(
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
            .request(
                LineRequestFlags::OUTPUT,
                GpioState::Deactivated.to_value(),
                GPIO_CONSUMER,
            )
            .with_context(|| format!("Failed to get access to GPIO {pin}"))?;

        Ok(Self {
            handle,
            activate_time,
            deactivate_time,
        })
    }

    fn next_state_and_timeout(&self, now: NaiveTime) -> Result<(GpioState, chrono::Duration)> {
        let until_on = match self.activate_time.signed_duration_since(now) {
            dur if dur < chrono::Duration::zero() => dur
                .checked_add(&chrono::Duration::days(1))
                .context("Failed to add day to until on duration")?,
            dur => dur,
        };

        let until_off = match self.deactivate_time.signed_duration_since(now) {
            dur if dur < chrono::Duration::zero() => dur
                .checked_add(&chrono::Duration::days(1))
                .context("Failed to add day to until off duration")?,
            dur => dur,
        };

        if until_on < until_off {
            return Ok((GpioState::Deactivated, until_on));
        }

        Ok((GpioState::Activated, until_off))
    }
}

#[async_trait]
impl Control for TimeBasedController {
    async fn run(&mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut timeout = Duration::from_secs(0);

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout)=> {
                    let now = Utc::now().time();
                    let (state, chrono_timeout) = self
                        .next_state_and_timeout(now)
                        .context("Failed to get next state and timout of control pin")?;

                    debug!("{} control pin", state.as_str());
                    self.handle
                        .set_value(state.to_value())
                        .context("Failed to set value of control pin")?;

                    debug!(
                        "{} control pin in {:02}:{:02}:{:02}h",
                        state.toggle().as_str(),
                        chrono_timeout.num_hours(),
                        chrono_timeout.num_minutes() % 60,
                        chrono_timeout.num_seconds() % 60
                    );

                    timeout = chrono_timeout
                        .to_std()
                        .context("Failed to convert chrono duration to std duration")?;

                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}

struct ThresholdController<M: ThresholdControl> {
    handle: LineHandle,
    receiver: broadcast::Receiver<Vec<M>>,
    threshold: M::Threshold,
}

impl<M: ThresholdControl> ThresholdController<M> {
    fn new(
        activate_condition: &str,
        deactivate_condition: &str,
        gpio_path: impl AsRef<Path>,
        pin: u32,
        receiver: broadcast::Receiver<Vec<M>>,
    ) -> Result<Self> {
        let threshold = M::threshold(activate_condition, deactivate_condition).context("")?;
        let mut chip = Chip::new(gpio_path).context("Failed to open GPIO chip")?;
        let handle = chip
            .get_line(pin)
            .with_context(|| format!("Failed to get handle to GPIO line {pin}"))?
            .request(
                LineRequestFlags::OUTPUT,
                GpioState::Deactivated.to_value(),
                GPIO_CONSUMER,
            )
            .with_context(|| format!("Failed to get access to GPIO {pin}"))?;

        Ok(Self {
            handle,
            receiver,
            threshold,
        })
    }
}

#[async_trait]
impl<M> Control for ThresholdController<M>
where
    M: ThresholdControl + Clone + Send,
    M::Threshold: Send,
{
    async fn run(&mut self, cancel_token: CancellationToken) -> Result<()> {
        loop {
            tokio::select! {
                value = self.receiver.recv() => {
                    match value {
                        Ok(measurements) => {
                            let state = M::desired_state(&measurements, &self.threshold)
                                .context("Failed to get desired state of control pin")?;
                            if let Some(s) = state {
                                self.handle
                                    .set_value(s.to_value())
                                    .context("Failed to set value of control pin")?;
                            }
                        },
                        Err(RecvError::Lagged(num_skipped)) => warn!("Skipping {num_skipped} measurements due to lagging"),
                        Err(RecvError::Closed) => {
                            if !cancel_token.is_cancelled() {
                                bail!("Failed to receive measurements: {}", RecvError::Closed)
                            }
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}
