use rppal::gpio::{Gpio, OutputPin, Pin};
use tokio::sync::oneshot;

use crate::app::LightControlArgs;

use super::error::ControlError;

pub struct LightController {
    args: LightControlArgs,
    pin: OutputPin,
    shutdown: oneshot::Receiver<()>,
}

impl LightController {
    pub fn start(args: LightControlArgs) -> Result<oneshot::Sender<()>, ControlError> {
        let (tx, rx) = oneshot::channel();
        if args.disable {
            log::info!("light controller is disabled by config arguments");
            return Ok(tx);
        }

        let gpio = Gpio::new().map_err(ControlError::InitGpioFailed)?;
        let pin = gpio
            .get(args.pin)
            .map_err(ControlError::GetPinFailed)?
            .into_output();

        tokio::spawn(
            Self {
                args,
                pin,
                shutdown: rx,
            }
            .run(),
        );

        Ok(tx)
    }

    pub async fn run(self) {
        loop {
            tokio::select! {

                _ = self.shutdown => {
                    log::debug!("light controller shutting down");
                    return;
                }
            }
        }
    }
}

fn check_args(args: &LightControlArgs) -> Result<(), ControlError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use chrono::{NaiveDateTime, NaiveTime, Utc};

    #[test]
    fn foo() {
        let now = Utc::now().time();
        let start = NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        let end = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let diff = end.sub(now);

        println!("now: {:?}", now);
        println!("{:?}", diff);
    }
}
