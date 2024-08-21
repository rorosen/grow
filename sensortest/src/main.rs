use std::{env, str::FromStr};

use anyhow::{bail, Context, Result};
use grow_measure::{
    air::{bme680::Bme680, AirSensor},
    light::{bh1750fvi::Bh1750Fvi, LightSensor},
    water_level::{vl53l0x::Vl53L0X, WaterLevelSensor},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
struct Config {
    address: u8,
    variant: Variant,
}

#[derive(Debug)]
enum Variant {
    Bme680,
    Bh1750Fvi,
    Vl53L0x,
}

impl FromStr for Variant {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bme680" => Ok(Self::Bme680),
            "bh1750fvi" => Ok(Self::Bh1750Fvi),
            "vl53l0x" => Ok(Self::Vl53L0x),
            arg => bail!("Unrecognized sensor model: {arg}"),
        }
    }
}

impl Config {
    fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let Some(name) = args.next() else {
            bail!("No program name");
        };

        let Some(variant) = args.next() else {
            print_usage(&name);
            bail!("No sensor model specified");
        };

        let Some(address) = args.next() else {
            print_usage(&name);
            bail!("No address specified");
        };

        let variant = Variant::from_str(&variant)?;
        let address = u8::from_str_radix(address.strip_prefix("0x").unwrap_or(&address), 16)
            .context("Failed to parse sensor address")?;

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
        Variant::Bme680 => {
            let mut sensor = Bme680::new(config.address).await.with_context(|| {
                format!("Failed to initialize BME680 sensor at {}", config.address)
            })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
        Variant::Bh1750Fvi => {
            let mut sensor = Vl53L0X::new(config.address).await.with_context(|| {
                format!(
                    "Failed to initialize BH1750FVI sensor at {}",
                    config.address
                )
            })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
        Variant::Vl53L0x => {
            let mut sensor = Bh1750Fvi::new(config.address).await.with_context(|| {
                format!("Failed to initialize VL53L0X sensor at {}", config.address)
            })?;
            let measurement = sensor.measure(token).await?;
            println!("{measurement:?}");
        }
    }

    Ok(())
}
