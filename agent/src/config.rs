use std::{fs::File, path::Path};

use air::AirConfig;
use air_pump::AirPumpControlConfig;
use anyhow::{Context, Result};
use fan::FanControlConfig;
use light::LightConfig;
use serde::{de::Error, Deserialize, Deserializer};
use water_level::WaterLevelConfig;

pub mod air;
pub mod air_pump;
pub mod fan;
pub mod light;
pub mod water_level;

#[derive(PartialEq, Debug, Deserialize)]
pub struct Config {
    pub light: LightConfig,
    pub water_level: WaterLevelConfig,
    pub fan: FanControlConfig,
    pub air: AirConfig,
    pub air_pump: AirPumpControlConfig,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path).context("failed to open config file")?;
        let config: Config = serde_json::from_reader(&file).context("failed to parse config")?;
        Ok(config)
    }
}

fn from_hex<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.strip_prefix("0x").unwrap_or(&s);
    u8::from_str_radix(s, 16).map_err(D::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    use air::{AirSampleConfig, ExhaustControlConfig, ExhaustControlMode};
    use chrono::NaiveTime;
    use light::{LightControlConfig, LightSampleConfig};
    use std::io::Write;
    use tempfile::NamedTempFile;
    use water_level::{PumpControlConfig, WaterLevelSampleConfig};

    #[test]
    fn parse_config_ok() {
        let mut file = NamedTempFile::new().unwrap();
        let input = serde_json::json!({
          "light": {
            "control": {
              "enable": true,
              "pin": 6,
              "activate_time": "10:00:00",
              "deactivate_time": "04:00:00"
            },
            "sample": {
              "left_address": "0x23",
              "right_address": "0x5C",
              "sample_rate_secs": 1800
            }
          },
          "water_level": {
            "control": {
              "enable": false,
              "pin": 17,
            },
            "sample": {
              "sensor_address": "0x29",
              "sample_rate_secs": 1800
            }
          },
          "fan": {
            "enable": true,
            "pin": 23,
            "on_duration_secs": 1,
            "off_duration_secs": 0
          },
          "air": {
            "control": {
              "mode": "Cyclic",
              "pin": 25,
              "on_duration_secs": 1,
              "off_duration_secs": 0
            },
            "sample": {
              "left_address": "0x77",
              "right_address": "0x76",
              "sample_rate_secs": 1800
            }
          },
          "air_pump": {
            "enable": true,
            "pin": 24
          }
        });
        let expected = Config {
            light: LightConfig {
                control: LightControlConfig {
                    enable: true,
                    pin: 6,
                    activate_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    deactivate_time: NaiveTime::from_hms_opt(04, 0, 0).unwrap(),
                },
                sample: LightSampleConfig {
                    left_address: 35,
                    right_address: 92,
                    sample_rate_secs: 1800,
                },
            },
            water_level: WaterLevelConfig {
                control: PumpControlConfig {
                    enable: false,
                    pin: 17,
                },
                sample: WaterLevelSampleConfig {
                    sensor_address: 41,
                    sample_rate_secs: 1800,
                },
            },
            fan: FanControlConfig {
                enable: true,
                pin: 23,
                on_duration_secs: 1,
                off_duration_secs: 0,
            },
            air: AirConfig {
                control: ExhaustControlConfig {
                    mode: ExhaustControlMode::Cyclic,
                    pin: 25,
                    on_duration_secs: 1,
                    off_duration_secs: 0,
                },
                sample: AirSampleConfig {
                    left_address: 119,
                    right_address: 118,
                    sample_rate_secs: 1800,
                },
            },
            air_pump: AirPumpControlConfig {
                enable: true,
                pin: 24,
            },
        };
        write!(&mut file, "{}", input.to_string()).unwrap();
        let config = Config::from_path(file.path()).expect("Can parse config file without error");

        assert_eq!(config, expected)
    }
}
