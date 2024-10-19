use std::{
    ops::{Add, Div},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio_util::sync::CancellationToken;

use crate::{
    control::{GpioState, ThresholdControl},
    threshold::{AirThreshold, Comparator, Threshold, WaterLevelThreshold},
};

// pub mod bh1750fvi;
// pub mod bme680;
mod feedback;
// mod i2c;
pub mod pylib;
// pub mod vl53l0x;

// #[trait_variant::make]
// pub trait Measure {
//     type Measurement;
//
//     async fn measure(&mut self, cancel_token: CancellationToken) -> Result<Self::Measurement>;
//     fn label(&self) -> &str;
// }

#[derive(Debug)]
pub enum AirField {
    Humidity,
    Pressure,
    Resistance,
    Temperature,
}

impl AirField {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Humidity => "humidity",
            Self::Pressure => "pressure",
            Self::Resistance => "resistance",
            Self::Temperature => "temperature",
        }
    }
}

impl FromStr for AirField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "humidity" => Ok(Self::Humidity),
            "pressure" => Ok(Self::Pressure),
            "resistance" => Ok(Self::Resistance),
            "temperature" => Ok(Self::Temperature),
            _ => bail!("unknown field: {s}"),
        }
    }
}

/// A single air measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
#[pyclass]
pub struct AirMeasurement {
    /// The number of seconds since unix epoch.
    #[pyo3(set)]
    pub measure_time: i64,
    /// The label of the sensor that took this measurement.
    #[pyo3(set)]
    pub label: String,
    /// The temperature in degree celsius.
    #[pyo3(set)]
    pub temperature: Option<f64>,
    /// The humidity in percentage.
    #[pyo3(set)]
    pub humidity: Option<f64>,
    /// The pressure in hectopascal.
    #[pyo3(set)]
    pub pressure: Option<f64>,
    /// The resistance due to Volatile Organic Compounds (VOC)
    /// and pollutants (except CO2) in the air.
    /// Higher concentration of VOCs leads to lower resistance.
    /// Lower concentration of VOCs leads to higher resistance.
    #[pyo3(set)]
    pub resistance: Option<f64>,
}

impl AirMeasurement {
    fn average_field(values: &[Self], field: &AirField) -> Result<f64> {
        let value = match field {
            AirField::Humidity => mean(values.iter().filter_map(|v| v.humidity)),
            AirField::Pressure => mean(values.iter().filter_map(|v| v.pressure)),
            AirField::Resistance => mean(values.iter().filter_map(|v| v.resistance)),
            AirField::Temperature => mean(values.iter().filter_map(|v| v.temperature)),
        };

        value.ok_or(anyhow!("field {:?} is none", field.as_str()))
    }
}

fn mean<T>(values: impl Iterator<Item = T>) -> Option<T>
where
    T: Add<Output = T> + From<u16> + Div<Output = T>,
{
    // start with 1 because reduce() takes first element as start
    let mut len: u16 = 1;
    let reduced = values.reduce(|acc, x| {
        len += 1;
        acc + x
    });
    reduced.map(|v| v / T::from(len))
}

impl ThresholdControl for AirMeasurement {
    type Threshold = AirThreshold;

    fn threshold(activate_condition: &str, deactivate_condition: &str) -> Result<Self::Threshold> {
        AirThreshold::new(activate_condition, deactivate_condition)
            .context("Failed to initialize air threshold")
    }

    fn desired_state(
        values: &[AirMeasurement],
        threshold: &Self::Threshold,
    ) -> Result<Option<GpioState>> {
        let ac = threshold.activate_condition();
        let dc = threshold.deactivate_condition();
        let activate_value = Self::average_field(values, ac.field())?;
        let deactivate_value = Self::average_field(values, dc.field())?;
        let is_exceeded = |thres, comp, val| match comp {
            Comparator::Lt => val < thres,
            Comparator::LtEq => val <= thres,
            Comparator::Gt => val > thres,
            Comparator::GtEq => val >= thres,
        };

        if is_exceeded(*dc.value(), dc.comparator(), deactivate_value) {
            return Ok(Some(GpioState::Deactivated));
        }

        if is_exceeded(*ac.value(), ac.comparator(), activate_value) {
            return Ok(Some(GpioState::Activated));
        }

        Ok(None)
    }
}

/// A single light measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
#[pyclass]
pub struct LightMeasurement {
    /// The number of seconds since unix epoch.
    #[pyo3(set)]
    pub measure_time: i64,
    /// The label of the sensor that took this measurement.
    #[pyo3(set)]
    pub label: String,
    /// The illuminance in lux.
    #[pyo3(set)]
    pub illuminance: Option<f64>,
}

#[derive(Debug)]
pub enum WaterLevelField {
    Distance,
}

impl WaterLevelField {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Distance => "distance",
        }
    }
}

impl FromStr for WaterLevelField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "distance" => Ok(Self::Distance),
            _ => bail!("unknown field: {s}"),
        }
    }
}

/// A single water level measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
#[pyclass]
pub struct WaterLevelMeasurement {
    /// The number of seconds since unix epoch.
    #[pyo3(set)]
    pub measure_time: i64,
    /// The label of the sensor that took this measurement.
    #[pyo3(set)]
    pub label: String,
    /// The distance between the sensor and the water surface in mm.
    #[pyo3(set)]
    pub distance: Option<u32>,
}

impl WaterLevelMeasurement {
    fn average_field(values: &[WaterLevelMeasurement], field: &WaterLevelField) -> Result<u32> {
        let value = match field {
            WaterLevelField::Distance => mean(values.iter().filter_map(|v| v.distance)),
        };

        value.ok_or(anyhow!("field {:?} is none", field.as_str()))
    }
}

impl ThresholdControl for WaterLevelMeasurement {
    type Threshold = WaterLevelThreshold;

    fn threshold(activate_condition: &str, deactivate_condition: &str) -> Result<Self::Threshold> {
        WaterLevelThreshold::new(activate_condition, deactivate_condition)
            .context("Failed to intialize water level threshold")
    }

    fn desired_state(
        values: &[WaterLevelMeasurement],
        threshold: &Self::Threshold,
    ) -> Result<Option<GpioState>> {
        let ac = threshold.activate_condition();
        let dc = threshold.deactivate_condition();
        let activate_value = Self::average_field(values, ac.field())?;
        let deactivate_value = Self::average_field(values, dc.field())?;
        let is_exceeded = |t, c, v| match c {
            Comparator::Lt => v < t,
            Comparator::LtEq => v <= t,
            Comparator::Gt => v > t,
            Comparator::GtEq => v >= t,
        };

        if is_exceeded(*dc.value(), dc.comparator(), deactivate_value) {
            return Ok(Some(GpioState::Deactivated));
        }

        if is_exceeded(*ac.value(), ac.comparator(), activate_value) {
            return Ok(Some(GpioState::Activated));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air_thresholds() -> (Vec<AirThreshold>, Vec<AirThreshold>, Vec<AirThreshold>) {
        let activate_thres = vec![
            AirThreshold::new("humidity >= 150", "humidity < 149.9").unwrap(),
            AirThreshold::new("pressure > 999.999", "pressure > 1000.0001").unwrap(),
            AirThreshold::new("humidity <= 150", "pressure >= 1000.0001").unwrap(),
            AirThreshold::new("temperature < 20.1", "temperature >= 100.0001").unwrap(),
            AirThreshold::new("resistance <= 400000", "resistance >= 500000").unwrap(),
        ];
        let deactivate_thres = vec![
            AirThreshold::new("humidity >= 150", "humidity < 190").unwrap(),
            AirThreshold::new("pressure < 999.999", "pressure >= 1000.000").unwrap(),
            AirThreshold::new("humidity <= 150", "pressure > 123.").unwrap(),
            AirThreshold::new("temperature >= 20.1", "temperature <= 100.0001").unwrap(),
        ];
        let neutral_thres = vec![
            AirThreshold::new("humidity > 150", "humidity < 149.9").unwrap(),
            AirThreshold::new("pressure > 1999.999", "pressure > 1000.0001").unwrap(),
            AirThreshold::new("humidity < 150", "pressure >= 1000.0001").unwrap(),
            AirThreshold::new("temperature < 10.1", "temperature >= 100.0001").unwrap(),
            AirThreshold::new("resistance <= 40000", "resistance >= 500000").unwrap(),
        ];

        (activate_thres, deactivate_thres, neutral_thres)
    }

    fn water_level_thresholds() -> (
        Vec<WaterLevelThreshold>,
        Vec<WaterLevelThreshold>,
        Vec<WaterLevelThreshold>,
    ) {
        let activate_thres = vec![
            WaterLevelThreshold::new("distance >= 250", "distance < 100").unwrap(),
            WaterLevelThreshold::new("distance < 251", "distance <= 200").unwrap(),
            WaterLevelThreshold::new("distance <= 2510", "distance >= 776").unwrap(),
            WaterLevelThreshold::new("distance > 201", "distance < 100").unwrap(),
        ];

        let deactivate_thres = vec![
            WaterLevelThreshold::new("distance >= 250", "distance < 300").unwrap(),
            WaterLevelThreshold::new("distance < 100", "distance <= 250").unwrap(),
        ];

        let neutral_thres = vec![
            WaterLevelThreshold::new("distance >= 2510", "distance >= 776").unwrap(),
            WaterLevelThreshold::new("distance > 251", "distance < 100").unwrap(),
        ];

        (activate_thres, deactivate_thres, neutral_thres)
    }

    #[test]
    fn desired_air_state_ok() {
        let meas = [
            AirMeasurement {
                measure_time: 0,
                label: String::new(),
                temperature: Some(10.),
                humidity: Some(100.),
                pressure: Some(1_000.),
                resistance: Some(10_000.),
            },
            AirMeasurement {
                measure_time: 0,
                label: String::new(),
                temperature: Some(20.),
                humidity: Some(200.),
                pressure: Some(1_000.),
                resistance: Some(100_000.),
            },
            AirMeasurement {
                measure_time: 0,
                label: String::new(),
                temperature: Some(30.),
                humidity: None,
                pressure: Some(1_000.),
                resistance: Some(1_000_000.),
            },
        ];

        let check_thresholds = |thresholds: &[AirThreshold], expected: Option<GpioState>| {
            for t in thresholds {
                assert_eq!(AirMeasurement::desired_state(&meas, t).unwrap(), expected);
            }
        };

        let thresholds = air_thresholds();
        check_thresholds(&thresholds.0, Some(GpioState::Activated));
        check_thresholds(&thresholds.1, Some(GpioState::Deactivated));
        check_thresholds(&thresholds.2, None);
    }

    #[test]
    fn desired_air_state_err() {
        let meas = [AirMeasurement {
            measure_time: 0,
            label: String::new(),
            temperature: None,
            humidity: None,
            pressure: None,
            resistance: None,
        }];

        let check_thresholds = |thresholds: &[AirThreshold]| {
            for t in thresholds {
                assert!(AirMeasurement::desired_state(&meas, t).is_err());
            }
        };

        let thresholds = air_thresholds();
        check_thresholds(&thresholds.0);
        check_thresholds(&thresholds.1);
        check_thresholds(&thresholds.2);
    }

    #[test]
    fn desired_water_level_state_ok() {
        let meas = [
            WaterLevelMeasurement {
                measure_time: 0,
                label: String::new(),
                distance: Some(200),
            },
            WaterLevelMeasurement {
                measure_time: 0,
                label: String::new(),
                distance: Some(300),
            },
            WaterLevelMeasurement {
                measure_time: 0,
                label: String::new(),
                distance: None,
            },
        ];

        let check_thresholds = |thresholds: &[WaterLevelThreshold], expected: Option<GpioState>| {
            for t in thresholds {
                assert_eq!(
                    WaterLevelMeasurement::desired_state(&meas, t).unwrap(),
                    expected
                );
            }
        };

        let thresholds = water_level_thresholds();
        check_thresholds(&thresholds.0, Some(GpioState::Activated));
        check_thresholds(&thresholds.1, Some(GpioState::Deactivated));
        check_thresholds(&thresholds.2, None);
    }

    #[test]
    fn desired_water_level_state_err() {
        let meas = [WaterLevelMeasurement {
            measure_time: 0,
            label: String::new(),
            distance: None,
        }];

        let check_thresholds = |thresholds: &[WaterLevelThreshold]| {
            for t in thresholds {
                assert!(WaterLevelMeasurement::desired_state(&meas, t).is_err());
            }
        };

        let thresholds = water_level_thresholds();
        check_thresholds(&thresholds.0);
        check_thresholds(&thresholds.1);
        check_thresholds(&thresholds.2);
    }
}
