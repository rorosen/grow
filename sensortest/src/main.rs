use std::{env, str::FromStr};

use anyhow::{bail, Context, Result};
use grow_measure::{air::AirSensor, light::LightSensor, water_level::WaterLevelSensor};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
struct Config {
    address: u8,
    variant: Variant,
}

#[derive(Debug)]
enum Variant {
    Air,
    Light,
    WaterLevel,
}

impl FromStr for Variant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "air" => Ok(Self::Air),
            "light" => Ok(Self::Light),
            "water" => Ok(Self::WaterLevel),
            arg => bail!("unrecognized measure variant: {arg}"),
        }
    }
}

impl Config {
    fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let Some(name) = args.next() else {
            bail!("no program name");
        };

        let Some(variant) = args.next() else {
            print_usage(&name);
            bail!("no variant passed");
        };

        let Some(address) = args.next() else {
            print_usage(&name);
            bail!("no address passed");
        };

        let variant = Variant::from_str(&variant)?;
        let address = u8::from_str_radix(address.strip_prefix("0x").unwrap_or(&address), 16)
            .context("failed to parse sensor address")?;

        Ok(Config { address, variant })
    }
}

fn print_usage(name: &str) {
    println!("Usage: {name} <variant> <sensor_address>");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::from_args(env::args())?;
    let token = CancellationToken::new();

    match &config.variant {
        Variant::Air => {
            let mut sensor = AirSensor::new(config.address).await.with_context(|| {
                format!("failed to initialize air sensor at {}", config.address)
            })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
        Variant::Light => {
            let mut sensor = LightSensor::new(config.address).await.with_context(|| {
                format!("failed to initialize light sensor at {}", config.address)
            })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
        Variant::WaterLevel => {
            let mut sensor = WaterLevelSensor::new(config.address)
                .await
                .with_context(|| {
                    format!(
                        "failed to initialize water level sensor at {}",
                        config.address
                    )
                })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
    }

    Ok(())
}
