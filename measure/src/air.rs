use crate::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio_util::sync::CancellationToken;

pub mod bme680;

#[async_trait]
pub trait AirSensor {
    async fn measure(
        &mut self,
        label: String,
        cancel_token: CancellationToken,
    ) -> Result<AirMeasurement, Error>;
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
    /// The resistance of the MOX sensor due to Volatile Organic Compounds (VOC)
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
