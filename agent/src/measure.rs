use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio_util::sync::CancellationToken;

pub mod bh1750fvi;
pub mod bme680;
mod i2c;
pub mod vl53l0x;

#[trait_variant::make]
pub trait Measure {
    type Measurement;

    async fn measure(&mut self, cancel_token: CancellationToken) -> Result<Self::Measurement>;
    fn label(&self) -> &str;
}

/// A single air measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct AirMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: String,
    /// The temperature in degree celsius.
    pub temperature: Option<f64>,
    /// The humidity in percentage.
    pub humidity: Option<f64>,
    /// The pressure in hectopascal.
    pub pressure: Option<f64>,
    /// The resistance due to Volatile Organic Compounds (VOC)
    /// and pollutants (except CO2) in the air.
    /// Higher concentration of VOCs leads to lower resistance.
    /// Lower concentration of VOCs leads to higher resistance.
    pub resistance: Option<f64>,
}

impl AirMeasurement {
    pub fn new(measure_time: i64, label: String) -> Self {
        Self {
            measure_time,
            label,
            temperature: None,
            humidity: None,
            pressure: None,
            resistance: None,
        }
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn humidity(mut self, humidity: f64) -> Self {
        self.humidity = Some(humidity);
        self
    }

    pub fn pressure(mut self, pressure: f64) -> Self {
        self.pressure = Some(pressure);
        self
    }

    pub fn resistance(mut self, resistance: f64) -> Self {
        self.resistance = Some(resistance);
        self
    }
}

// impl Storable<'_> for AirMeasurement {
//     fn into_insert_query<'args>(
//         &self,
//     ) -> sqlx::query::Query<'args, sqlx::Sqlite, <sqlx::Sqlite as sqlx::Database>::Arguments<'args>>
//     {
//         let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
//             "INSERT INTO air_measurements(measure_time, label, temperature, humidity, pressure, resistance) ",
//         );
//         query_builder
//             .push_bind(self.measure_time)
//             .push_bind(self.label)
//             .push_bind(self.temperature)
//             .push_bind(self.humidity)
//             .push_bind(self.pressure)
//             .push_bind(self.resistance);
//
//         query_builder.build()
//     }
// }

/// A single light measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct LightMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: String,
    /// The illuminance in lux.
    pub illuminance: Option<f64>,
}

impl LightMeasurement {
    pub fn new(measure_time: i64, label: String) -> Self {
        Self {
            measure_time,
            label,
            illuminance: None,
        }
    }

    pub fn illuminance(mut self, illuminance: f64) -> Self {
        self.illuminance = Some(illuminance);
        self
    }
}

// impl Storable<'_> for LightMeasurement {
//     fn into_insert_query<'args>(
//         &self,
//     ) -> sqlx::query::Query<'args, sqlx::Sqlite, <sqlx::Sqlite as sqlx::Database>::Arguments<'args>>
//     {
//         let mut query_builder: QueryBuilder<Sqlite> =
//             QueryBuilder::new("INSERT INTO light_measurements(measure_time, label, illuminance) ");
//         query_builder
//             .push_bind(self.measure_time)
//             .push_bind(self.label)
//             .push_bind(self.illuminance);
//
//         query_builder.build()
//     }
// }

/// A single water level measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct WaterLevelMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: i64,
    /// The label of this measurement, used to organize measurements.
    pub label: String,
    /// The distance between the sensor and the water surface in mm.
    pub distance: Option<u32>,
}

impl WaterLevelMeasurement {
    pub fn new(measure_time: i64, label: String) -> Self {
        Self {
            measure_time,
            label,
            distance: None,
        }
    }

    pub fn distance(mut self, distance: u32) -> Self {
        self.distance = Some(distance);
        self
    }
}

// impl Storable<'_> for WaterLevelMeasurement {
//     fn into_insert_query<'args>(
//         &self,
//     ) -> sqlx::query::Query<'args, sqlx::Sqlite, <sqlx::Sqlite as sqlx::Database>::Arguments<'args>>
//     {
//         let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
//             "INSERT INTO water_level_measurements(measure_time, label, distance) ",
//         );
//         query_builder
//             .push_bind(self.measure_time)
//             .push_bind(self.label)
//             .push_bind(self.distance);
//
//         query_builder.build()
//     }
// }
