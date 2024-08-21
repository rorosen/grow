use std::env;

use anyhow::{bail, Context, Result};
use rppal::gpio::{Gpio, Level};

#[derive(Debug)]
struct Config {
    pin: u8,
    level: Level,
}

impl Config {
    fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let Some(name) = args.next() else {
            bail!("No program name");
        };

        let Some(pin) = args.next() else {
            print_usage(&name);
            bail!("No pin number passed");
        };

        let Some(level) = args.next() else {
            print_usage(&name);
            bail!("No level (low, high) passed");
        };

        let pin = u8::from_str_radix(&pin, 10).context("Failed to parse pin number")?;
        let level = match level.to_lowercase().as_str() {
            "low" => Level::Low,
            "high" => Level::High,
            arg => bail!("Unrecognized level: {arg}"),
        };

        Ok(Config { pin, level })
    }
}

fn print_usage(name: &str) {
    println!("Usage: {name} <pin> <level>");
}

fn main() -> Result<(), anyhow::Error> {
    let config = Config::from_args(env::args())?;
    let mut pin = Gpio::new()
        .context("Failed to construct GPIO instance")?
        .get(config.pin)
        .context("Failed to get GPIO pin")?
        .into_output();

    pin.set_reset_on_drop(false);
    match config.level {
        Level::Low => pin.set_low(),
        Level::High => pin.set_high(),
    }

    Ok(())
}
