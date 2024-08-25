use std::env;

use anyhow::{bail, Context, Result};
use gpio_cdev::{Chip, LineRequestFlags};

#[derive(Debug)]
struct Config {
    name: String,
    chip_path: String,
    line: u32,
    value: u8,
}

impl Config {
    fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let chip_path =
            env::var("GROW_GPIOCHIP").unwrap_or_else(|_| String::from("/dev/gpiochip0"));

        let Some(name) = args.next() else {
            bail!("No program name");
        };

        let Some(line) = args.next() else {
            print_usage(&name);
            bail!("No line/pin number passed");
        };

        let Some(level) = args.next() else {
            print_usage(&name);
            bail!("No level (low, high) passed");
        };

        let line = line.parse().context("Failed to parse pin number")?;
        let value = match level.to_lowercase().as_str() {
            "low" => 0,
            "high" => 1,
            arg => bail!("Unrecognized level: {arg}"),
        };

        Ok(Config {
            name,
            chip_path,
            line,
            value,
        })
    }
}

fn print_usage(name: &str) {
    println!("Usage: {name} <pin> <level>");
}

fn main() -> Result<(), anyhow::Error> {
    let config = Config::from_args(env::args())?;
    let mut chip = Chip::new(config.chip_path)?;

    chip.get_line(config.line)?
        .request(LineRequestFlags::OUTPUT, config.value, &config.name)?;

    // println!("Output being driven... Enter to exit");
    // let mut buf = String::new();
    // ::std::io::stdin().read_line(&mut buf)?;

    Ok(())
}
