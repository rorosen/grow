use std::{
    fs::File,
    path::{Path, PathBuf},
};

use air::AirConfig;
use air_pump::AirPumpConfig;
use anyhow::{Context, Result};
use fan::FanConfig;
use light::LightConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use water_level::WaterLevelConfig;

pub mod air;
pub mod air_pump;
pub mod fan;
pub mod light;
pub mod water_level;
pub mod control;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_i2c_path")]
    pub i2c_path: PathBuf,
    #[serde(default = "default_gpio_path")]
    pub gpio_path: PathBuf,
    #[serde(default = "default_grow_id")]
    pub grow_id: String,
    #[serde(default)]
    pub air: AirConfig,
    #[serde(default)]
    pub air_pump: AirPumpConfig,
    #[serde(default)]
    pub fan: FanConfig,
    #[serde(default)]
    pub light: LightConfig,
    #[serde(default)]
    pub water_level: WaterLevelConfig,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(&path).context("Failed to open config file")?;
        let config: Config = serde_json::from_reader(&file).context("Failed to parse config")?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            i2c_path: default_i2c_path(),
            gpio_path: default_gpio_path(),
            grow_id: default_grow_id(),
            air: AirConfig::default(),
            air_pump: AirPumpConfig::default(),
            fan: FanConfig::default(),
            light: LightConfig::default(),
            water_level: WaterLevelConfig::default(),
        }
    }
}

fn default_i2c_path() -> PathBuf {
    "/dev/i2c-1".into()
}

fn default_gpio_path() -> PathBuf {
    "/dev/gpiochip0".into()
}

fn default_grow_id() -> String {
    "grow".into()
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

    use air::{AirSampleConfig, AirSensorConfig, AirSensorModel};
    use chrono::NaiveTime;
    use control::ControlConfig;
    use light::{ LightSampleConfig, LightSensorConfig, LightSensorModel};
    use std::{collections::HashMap, io::Write};
    use tempfile::NamedTempFile;
    use water_level::{
        WaterLevelSampleConfig, WaterLevelSensorConfig,
        WaterLevelSensorModel,
    };

    #[test]
    fn parse_config_ok() {
        let mut file = NamedTempFile::new().expect("Should be able to create tempfile");
        let input = serde_json::json!({
            "i2c_path": "/dev/i2c-69",
            "gpio_path": "/dev/gpiochip69",
            "grow_id": "tomatoes",
            "air": {
                "control": {
                    "mode": "Cyclic",
                    "pin": 25,
                    "on_duration_secs": 1,
                    "off_duration_secs": 0
                },
                "sample": {
                    "sample_rate_secs": 1800,
                    "sensors": {
                        "left": {
                            "model": "Bme680",
                            "address": "0x77"
                        },
                        "right": {
                            "model": "Bme680",
                            "address": "0x76"
                        }
                    }
                }
            },
            "air_pump": {
                "control": {
                    "mode": "Cyclic",
                    "pin": 24,
                    "on_duration_secs": 1,
                    "off_duration_secs": 0
                },
            },
            "fan": {
                "control": {
                    "mode": "Cyclic",
                    "pin": 23,
                    "on_duration_secs": 0,
                    "off_duration_secs": 1
                },
            },
            "light": {
                "control": {
                    "mode": "TimeBased",
                    "pin": 6,
                    "activate_time": "10:00:00",
                    "deactivate_time": "04:00:00"
                },
                "sample": {
                    "sample_rate_secs": 123,
                    "sensors": {
                        "left": {
                            "model": "Bh1750Fvi",
                            "address": "0x23"
                        },
                        "right": {
                            "model": "Bh1750Fvi",
                            "address": "0x5C"
                        }
                    }
                }
            },
            "water_level": {
                "control": {
                    "mode": "Feedback",
                    "pin": 17,
                    "activate_condition": "distance > 1",
                    "deactivate_condition": "distance > 9"
                },
                "sample": {
                    "sample_rate_secs": 86400,
                    "sensors": {
                        "main": {
                            "model": "Vl53L0X",
                            "address": "0x29"
                        }
                    }
                }
            }
        });

        let expected = Config {
            i2c_path: PathBuf::from("/dev/i2c-69"),
            gpio_path: PathBuf::from("/dev/gpiochip69"),
            grow_id: String::from("tomatoes"),
            air: AirConfig {
                control: ControlConfig::Cyclic {
                    pin: 25,
                    on_duration_secs: 1,
                    off_duration_secs: 0,
                },
                sample: AirSampleConfig {
                    sample_rate_secs: 1800,
                    sensors: HashMap::from([
                        (
                            "left".into(),
                            AirSensorConfig {
                                model: AirSensorModel::Bme680,
                                address: 119,
                            },
                        ),
                        (
                            "right".into(),
                            AirSensorConfig {
                                model: AirSensorModel::Bme680,
                                address: 118,
                            },
                        ),
                    ]),
                },
            },
            air_pump: AirPumpConfig {
                control: ControlConfig::Cyclic {
                    pin: 24,
                    on_duration_secs: 1,
                    off_duration_secs: 0,
                },
            },
            fan: FanConfig {
                control: ControlConfig::Cyclic {
                    pin: 23,
                    on_duration_secs: 0,
                    off_duration_secs: 1,
                },
            },
            light: LightConfig {
                control: ControlConfig::TimeBased {
                    pin: 6,
                    activate_time: NaiveTime::from_hms_opt(10, 0, 0)
                        .expect("Failed to craete NaiveTime"),
                    deactivate_time: NaiveTime::from_hms_opt(4, 0, 0)
                        .expect("Failed to craete NaiveTime"),
                },
                sample: LightSampleConfig {
                    sample_rate_secs: 123,
                    sensors: HashMap::from([
                        (
                            "left".into(),
                            LightSensorConfig {
                                model: LightSensorModel::Bh1750Fvi,
                                address: 35,
                            },
                        ),
                        (
                            "right".into(),
                            LightSensorConfig {
                                model: LightSensorModel::Bh1750Fvi,
                                address: 92,
                            },
                        ),
                    ]),
                },
            },
            water_level: WaterLevelConfig {
                control: ControlConfig::Feedback {
                    pin: 17,
                    activate_condition: String::from("distance > 1"),
                    deactivate_condition: String::from("distance > 9"),
                },
                sample: WaterLevelSampleConfig {
                    sample_rate_secs: 86400,
                    sensors: HashMap::from([(
                        "main".into(),
                        WaterLevelSensorConfig {
                            model: WaterLevelSensorModel::Vl53L0X,
                            address: 41,
                        },
                    )]),
                },
            },
        };
        write!(&mut file, "{input}").expect("Tempfile should be writable");
        let config =
            Config::from_file(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }

    #[test]
    fn parse_empty_config_ok() {
        let mut file = NamedTempFile::new().expect("should be able to create tempfile");
        write!(&mut file, "{{}}").expect("tempfile should be writable");
        let config = Config::from_file(file.path())
            .expect("Empty config file should be parsed without error");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn parse_partial_config_ok() {
        let mut file = NamedTempFile::new().expect("Should be able to create tempfile");
        let input = serde_json::json!({
            "light": {
                "control": {
                    "mode": "TimeBased",
                    "pin": 6,
                    "activate_time": "10:00:00",
                    "deactivate_time": "04:00:00"
                },
                "sample": {
                    "sample_rate_secs": 123,
                    "sensors": {
                        "left": {
                            "model": "Bh1750Fvi",
                            "address": "0x23"
                        },
                        "right": {
                            "model": "Bh1750Fvi",
                            "address": "0x5C"
                        }
                    }
                }
            }
        });

        let expected = Config {
            light: LightConfig {
                control: ControlConfig::TimeBased {
                    pin: 6,
                    activate_time: NaiveTime::from_hms_opt(10, 0, 0)
                        .expect("Failed to craete NaiveTime"),
                    deactivate_time: NaiveTime::from_hms_opt(4, 0, 0)
                        .expect("Failed to craete NaiveTime"),
                },
                sample: LightSampleConfig {
                    sample_rate_secs: 123,
                    sensors: HashMap::from([
                        (
                            "left".into(),
                            LightSensorConfig {
                                model: LightSensorModel::Bh1750Fvi,
                                address: 35,
                            },
                        ),
                        (
                            "right".into(),
                            LightSensorConfig {
                                model: LightSensorModel::Bh1750Fvi,
                                address: 92,
                            },
                        ),
                    ]),
                },
            },
            ..Default::default()
        };
        write!(&mut file, "{}", input).expect("Tempfile should be writable");
        let config =
            Config::from_file(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }
}
