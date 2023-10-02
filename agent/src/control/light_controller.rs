use chrono::{NaiveTime, Utc};
use clap::Parser;
use rppal::gpio::{Gpio, OutputPin};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::error::Error;

#[derive(Debug, Parser)]
pub struct LightControlArgs {
    /// Whether to disable the light controller
    #[arg(
        id = "light_control_disable",
        long = "light-control-disable",
        env = "GROW_AGENT_LIGHT_CONTROL_DISABLE"
    )]
    pub disable: bool,

    /// The gpio pin used to disable the light
    #[arg(
        id = "light_control_pin",
        long = "light-control-pin",
        env = "GROW_AGENT_LIGHT_CONTROL_PIN",
        default_value_t = 6
    )]
    pub pin: u8,

    /// The time of the day when the light should be switched on
    #[arg(
        id = "light_control_switch_on_hour",
        long = "light-control-switch-on-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_HOUR",
        default_value_t = NaiveTime::from_hms_opt(10, 0, 0).unwrap()
    )]
    pub activate_time: NaiveTime,

    /// The time of the day when the light should be switched off
    #[arg(
        id = "light_control_switch_off_hour",
        long = "light-control-switch-off-hour",
        env = "GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_HOUR",
        default_value_t = NaiveTime::from_hms_opt(22, 0, 0).unwrap()
    )]
    pub deactivate_time: NaiveTime,
}

pub struct LightController {
    pin: OutputPin,
    cancel_token: CancellationToken,
    activate_time: NaiveTime,
    deactivate_time: NaiveTime,
}

impl LightController {
    pub fn start(
        args: LightControlArgs,
        cancel_token: CancellationToken,
        finish: mpsc::Sender<()>,
    ) -> Result<(), Error> {
        if args.disable {
            log::info!("light controller is disabled by configuration");
            return Ok(());
        }

        if args.activate_time == args.deactivate_time {
            return Err(Error::InvalidArgs(
                "light".into(),
                "activate time and deactivate time cannot be equal".into(),
            ));
        }

        let gpio = Gpio::new().map_err(Error::InitGpioFailed)?;
        let pin = gpio
            .get(args.pin)
            .map_err(Error::GetPinFailed)?
            .into_output();

        tokio::spawn(
            Self {
                pin,
                cancel_token,
                activate_time: args.activate_time,
                deactivate_time: args.deactivate_time,
            }
            .run(finish),
        );

        Ok(())
    }

    pub async fn run(mut self, _finish: mpsc::Sender<()>) {
        log::debug!("starting light controller");
        let mut timeout = chrono::Duration::zero();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(timeout.to_std().unwrap()) => {
                    timeout = self.control();
                }
                _ = self.cancel_token.cancelled() => {
                    log::debug!("light controller shutting down");
                    return;
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
