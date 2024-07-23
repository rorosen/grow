use std::{fs::File, path::Path};

use air::AirConfig;
use air_pump::AirPumpControlConfig;
use anyhow::{Context, Result};
use fan::FanControlConfig;
use light::LightConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use water_level::WaterLevelConfig;

pub mod air;
pub mod air_pump;
pub mod fan;
pub mod light;
pub mod water_level;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub light: LightConfig,
    pub water_level: WaterLevelConfig,
    pub fan: FanControlConfig,
    pub air: AirConfig,
    pub air_pump: AirPumpControlConfig,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path).context("failed to open config file")?;
        let config: Config = serde_json::from_reader(&file).context("failed to parse config")?;
        Ok(config)
    }
}

fn from_hex<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let s = s.strip_prefix("0x").unwrap_or(s);
    u8::from_str_radix(&s, 16).map_err(D::Error::custom)
}
