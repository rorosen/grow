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

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub light: LightConfig,
    #[serde(default)]
    pub water_level: WaterLevelConfig,
    #[serde(default)]
    pub fan: FanControlConfig,
    #[serde(default)]
    pub air: AirConfig,
    #[serde(default)]
    pub air_pump: AirPumpControlConfig,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path).context("Failed to open config file")?;
        let config: Config = serde_json::from_reader(&file).context("Failed to parse config")?;
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

    use air::{
        AirSampleConfig, AirSensorConfig, AirSensorModel, ExhaustControlConfig, ExhaustControlMode,
    };
    use chrono::NaiveTime;
    use light::{LightControlConfig, LightSampleConfig, LightSensorConfig, LightSensorModel};
    use std::{collections::HashMap, io::Write};
    use tempfile::NamedTempFile;
    use water_level::{
        PumpControlConfig, WaterLevelSampleConfig, WaterLevelSensorConfig, WaterLevelSensorModel,
    };

    #[test]
    fn parse_config_ok() {
        let mut file = NamedTempFile::new().expect("should be able to create tempfile");
        let input = serde_json::json!({
            "light": {
                "control": {
                    "enable": true,
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
                    "enable": false,
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
                control: PumpControlConfig {
                    enable: false,
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
            air_pump: AirPumpControlConfig {
                enable: true,
                pin: 24,
            },
        };
        write!(&mut file, "{}", input.to_string()).expect("tempfile should be writable");
        let config =
            Config::from_path(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }

    #[test]
    fn parse_empty_config_ok() {
        let mut file = NamedTempFile::new().expect("should be able to create tempfile");
        write!(&mut file, "{{}}").expect("tempfile should be writable");
        let config = Config::from_path(file.path())
            .expect("Empty config file should be parsed without error");
        assert_eq!(config, Config::default());
    }

    #[test]
    fn parse_partial_config_ok() {
        let mut file = NamedTempFile::new().expect("should be able to create tempfile");
        let input = serde_json::json!({
            "light": {
                "control": {
                    "enable": true,
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
                    enable: true,
                    pin: 6,
                    activate_time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                    deactivate_time: NaiveTime::from_hms_opt(04, 0, 0).unwrap(),
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
        write!(&mut file, "{}", input.to_string()).expect("tempfile should be writable");
        let config =
            Config::from_path(file.path()).expect("Config file should be parsed without error");
        assert_eq!(config, expected)
    }
}
