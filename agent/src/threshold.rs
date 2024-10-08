use std::{fmt::Debug, str::FromStr};

use anyhow::{bail, Context, Result};
use pest::Parser;
use pest_derive::Parser;

use crate::measure::{AirField, WaterLevelField};

#[derive(Parser)]
#[grammar = "threshold.pest"]
struct ConditionParser;

pub trait Threshold
where
    Self::Field: Debug,
    Self::Value: PartialOrd + Debug,
{
    type Field;
    type Value;

    fn activate_condition(&self) -> &Condition<Self::Field, Self::Value>;
    fn deactivate_condition(&self) -> &Condition<Self::Field, Self::Value>;
}

#[derive(Debug, Clone, Copy)]
pub enum Comparator {
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl FromStr for Comparator {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::LtEq),
            ">" => Ok(Self::Gt),
            ">=" => Ok(Self::GtEq),
            _ => bail!("unknown comparator: {value}"),
        }
    }
}

#[derive(Debug)]
pub struct Condition<F: Debug, V: Debug> {
    field: F,
    comparator: Comparator,
    value: V,
}

impl<F: Debug, V: Debug> Condition<F, V> {
    pub fn field(&self) -> &F {
        &self.field
    }

    pub fn comparator(&self) -> Comparator {
        self.comparator
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<F, V> TryFrom<&str> for Condition<F, V>
where
    F: FromStr<Err = anyhow::Error> + Debug,
    V: FromStr + Debug,
    <V as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut condition = ConditionParser::parse(Rule::condition, value)
            .context("Failed to parse condition")?
            .next()
            .expect("Never fails according to pest")
            .into_inner();

        let field = condition.next().expect("Every condition has a field");
        let comparator = condition.next().expect("Every condition has a comparator");
        let value = condition.next().expect("Every condition has a value");

        let field = F::from_str(field.as_str()).context("Failed to parse field")?;
        let comparator =
            Comparator::from_str(comparator.as_str()).context("Failed to parse comparator")?;
        let value = V::from_str(value.as_str()).context("Failed to parse value")?;

        Ok(Self {
            field,
            comparator,
            value,
        })
    }
}

#[derive(Debug)]
pub struct AirThreshold {
    activate_condition: Condition<AirField, f64>,
    deactivate_condition: Condition<AirField, f64>,
}

impl AirThreshold {
    pub fn new(activate_condition: &str, deactivate_condition: &str) -> Result<Self> {
        let activate_condition = Condition::try_from(activate_condition)
            .context("Failed to parse activate condition")?;
        let deactivate_condition = Condition::try_from(deactivate_condition)
            .context("Failed to parse deactivate condition")?;

        Ok(Self {
            activate_condition,
            deactivate_condition,
        })
    }
}

impl Threshold for AirThreshold {
    type Field = AirField;
    type Value = f64;

    fn activate_condition(&self) -> &Condition<Self::Field, Self::Value> {
        &self.activate_condition
    }

    fn deactivate_condition(&self) -> &Condition<Self::Field, Self::Value> {
        &self.deactivate_condition
    }
}

#[derive(Debug)]
pub struct WaterLevelThreshold {
    activate_condition: Condition<WaterLevelField, u32>,
    deactivate_condition: Condition<WaterLevelField, u32>,
}

impl WaterLevelThreshold {
    pub fn new(activate_condition: &str, deactivate_condition: &str) -> Result<Self> {
        let activate_condition = Condition::try_from(activate_condition)
            .context("Failed to parse activate condition")?;
        let deactivate_condition = Condition::try_from(deactivate_condition)
            .context("Failed to parse deactivate condition")?;

        Ok(Self {
            activate_condition,
            deactivate_condition,
        })
    }
}

impl Threshold for WaterLevelThreshold {
    type Field = WaterLevelField;
    type Value = u32;

    fn activate_condition(&self) -> &Condition<Self::Field, Self::Value> {
        &self.activate_condition
    }

    fn deactivate_condition(&self) -> &Condition<Self::Field, Self::Value> {
        &self.deactivate_condition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_air_threshold_ok() {
        let runs = vec![
            ("humidity >= 60.3", "humidity < 42.8"),
            ("pressure > 1100", "pressure <= 1010.6"),
            ("resistance <= 31000.", "resistance >= 45000"),
            ("temperature > 26.3", "temperature < 21."),
        ];

        for run in runs {
            AirThreshold::new(run.0, run.1).unwrap();
        }
    }

    #[test]
    fn parse_air_threshold_err() {
        let runs = vec![
            ("", "humidity < 42.8"),
            (".3", "humidity < 42.8"),
            ("pressure > 1100", "foo <= 1010.6"),
            ("resistance <= abc.", "resistance >= 45000"),
            ("temperature", "temperature < 21."),
            ("temperature > ", "temperature < 21."),
            ("temperature > 3", "temperature < 21. < humidity"),
        ];

        for run in runs {
            AirThreshold::new(run.0, run.1).unwrap_err();
        }
    }

    #[test]
    fn parse_water_level_threshold_ok() {
        let runs = vec![
            ("distance > 12", "distance < 4"),
            ("distance>=11", "distance<=7"),
        ];

        for run in runs {
            WaterLevelThreshold::new(run.0, run.1).unwrap();
        }
    }

    #[test]
    fn parse_water_level_threshold_err() {
        let runs = vec![
            ("distanc > 12", "distance < 4"),
            ("distance11", "distance<=7"),
        ];

        for run in runs {
            WaterLevelThreshold::new(run.0, run.1).unwrap_err();
        }
    }
}
