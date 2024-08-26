use std::{
    fs::File,
    path::{Path, PathBuf},
};

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

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_i2c_path")]
    pub i2c_path: PathBuf,
    #[serde(default = "default_gpio_path")]
    pub gpio_path: PathBuf,
    #[serde(default)]
    pub air: AirConfig,
    #[serde(default)]
    pub air_pump_control: AirPumpControlConfig,
    #[serde(default)]
    pub fan: FanControlConfig,
    #[serde(default)]
    pub light: LightConfig,
    #[serde(default)]
    pub water_level: WaterLevelConfig,
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let config_exists = path
            .as_ref()
            .try_exists()
            .context("Failed to check existence of config file")?;

        if !config_exists {
            let file = File::create(&path).context("Failed to create config file")?;
            let config = Self::default();
            serde_json::to_writer(file, &config).context("Failed to write default config")?;
            Ok(config)
        } else {
            let file = File::open(&path).context("Failed to open config file")?;
            let config: Config =
                serde_json::from_reader(&file).context("Failed to parse config")?;
            Ok(config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            i2c_path: default_i2c_path(),
            gpio_path: default_gpio_path(),
            air: AirConfig::default(),
            air_pump_control: AirPumpControlConfig::default(),
            fan: FanControlConfig::default(),
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

    use air::{
        AirSampleConfig, AirSensorConfig, AirSensorModel, ExhaustControlConfig, ExhaustControlMode,
    };
    use air_pump::AirPumpControlMode;
    use chrono::NaiveTime;
    use fan::FanControlMode;
    use light::{
        LightControlConfig, LightControlMode, LightSampleConfig, LightSensorConfig,
        LightSensorModel,
    };
    use std::{collections::HashMap, io::Write};
    use tempfile::{tempdir, NamedTempFile};
    use water_level::{
        WaterLevelControlConfig, WaterLevelControlMode, WaterLevelSampleConfig,
        WaterLevelSensorConfig, WaterLevelSensorModel,
    };

    #[test]
    fn parse_config_ok() {
        let mut file = NamedTempFile::new().expect("Should be able to create tempfile");
        let input = serde_json::json!({
            "i2c_path": "/dev/i2c-69",
            "gpio_path": "/dev/gpiochip69",
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
                    "mode": "Off",
                    "pumps": {
                        "main": 17
                    }
                },
                "sample": {
                    "sample_rate_secs": 86400,
                    "sensors": {
                        "main": {
                            "model": "Vl53L0x",
                            "address": "0x29"
                        }
                    }
                }
            },
            "fan": {
                "mode": "Cyclic",
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
            "air_pump_control": {
                "mode": "Permanent",
                "pin": 24
            }
        });

        let expected = Config {
            i2c_path: PathBuf::from("/dev/i2c-69"),
            gpio_path: PathBuf::from("/dev/gpiochip69"),
            light: LightConfig {
                control: LightControlConfig {
                    mode: LightControlMode::TimeBased,
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
                control: WaterLevelControlConfig {
                    mode: WaterLevelControlMode::Off,
                    pumps: HashMap::from([("main".into(), 17)]),
                },
                sample: WaterLevelSampleConfig {
                    sample_rate_secs: 86400,
                    sensors: HashMap::from([(
                        "main".into(),
                        WaterLevelSensorConfig {
                            model: WaterLevelSensorModel::Vl53L0x,
                            address: 41,
                        },
                    )]),
                },
            },
            fan: FanControlConfig {
                mode: FanControlMode::Cyclic,
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
            air_pump_control: AirPumpControlConfig {
                mode: AirPumpControlMode::Permanent,
                pin: 24,
            },
        };
        write!(&mut file, "{input}").expect("Tempfile should be writable");
        let config = Config::new(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }

    #[test]
    fn parse_empty_config_ok() {
        let dir = tempdir().expect("Temporary directory should be created");
        let config =
            Config::new(dir.path().join("config.json")).expect("Default config should be created");
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
                control: LightControlConfig {
                    mode: LightControlMode::TimeBased,
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
        let config = Config::new(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }
}
