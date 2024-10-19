use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use control::ControlConfig;
use sample::SampleConfig;
use serde::{Deserialize, Serialize};

pub mod control;
pub mod sample;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: SampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirPumpConfig {
    #[serde(default)]
    pub control: ControlConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct FanConfig {
    #[serde(default)]
    pub control: ControlConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct LightConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: SampleConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct WaterLevelConfig {
    #[serde(default)]
    pub control: ControlConfig,
    #[serde(default)]
    pub sample: SampleConfig,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveTime;
    use control::ControlConfig;
    use sample::SampleConfig;
    use std::{io::Write, time::Duration};
    use tempfile::NamedTempFile;

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
                    "on_duration": "1s",
                    "off_duration": "0"
                },
                "sample": {
                    "mode": "Interval",
                    "period": "30m",
                    "script_path": "/foo/bar/script.py",
                }
            },
            "air_pump": {
                "control": {
                    "mode": "Cyclic",
                    "pin": 24,
                    "on_duration": "1m30s",
                    "off_duration": "2h"
                },
            },
            "fan": {
                "control": {
                    "mode": "Cyclic",
                    "pin": 23,
                    "on_duration": "100s",
                    "off_duration": "10d"
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
                    "mode": "Off",
                }
            },
            "water_level": {
                "control": {
                    "mode": "Feedback",
                    "pin": 17,
                    "activate_condition": "distance > 8",
                    "deactivate_condition": "distance < 3"
                },
                "sample": {
                    "mode": "Interval",
                    "period": "1h",
                    "script_path": "/path/baz/another_script.py",
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
                    on_duration: Duration::from_secs(1),
                    off_duration: Duration::from_secs(0),
                },
                sample: SampleConfig::Interval {
                    period: Duration::from_secs(1800),
                    script_path: PathBuf::from("/foo/bar/script.py"),
                },
            },
            air_pump: AirPumpConfig {
                control: ControlConfig::Cyclic {
                    pin: 24,
                    on_duration: Duration::from_secs(90),
                    off_duration: Duration::from_secs(2 * 3600),
                },
            },
            fan: FanConfig {
                control: ControlConfig::Cyclic {
                    pin: 23,
                    on_duration: Duration::from_secs(100),
                    off_duration: Duration::from_secs(10 * 24 * 3600),
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
                sample: SampleConfig::Off,
            },
            water_level: WaterLevelConfig {
                control: ControlConfig::Feedback {
                    pin: 17,
                    activate_condition: String::from("distance > 8"),
                    deactivate_condition: String::from("distance < 3"),
                },
                sample: SampleConfig::Interval {
                    period: Duration::from_secs(3600),
                    script_path: PathBuf::from("/path/baz/another_script.py"),
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
                    "mode": "Interval",
                    "period": "2h+30m",
                    "script_path": "/some_script.py",
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
                sample: SampleConfig::Interval {
                    period: Duration::from_secs(2 * 3600 + 1800),
                    script_path: PathBuf::from("/some_script.py"),
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
