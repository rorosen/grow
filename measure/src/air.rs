use std::time::SystemTime;

use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

pub mod bme680;

#[async_trait]
pub trait AirSensor {
    async fn measure(&mut self, cancel_token: CancellationToken) -> Result<AirMeasurement, Error>;
}

/// A single air measurement.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AirMeasurement {
    /// The number of seconds since unix epoch.
    pub measure_time: u64,
    /// The label of this measurement, used to organize measurements.
    pub label: Option<String>,
    /// The temperature in degree celsius.
    pub temperature: f64,
    /// The humidity in percentage.
    pub humidity: f64,
    /// The pressure in hectopascal.
    pub pressure: f64,
    /// The resistance of the MOX sensor due to Volatile Organic Compounds (VOC)
    /// and pollutants (except CO2) in the air.
    /// Higher concentration of VOCs leads to lower resistance.
    /// Lower concentration of VOCs leads to higher resistance.
    pub resistance: Option<f64>,
}

impl AirMeasurement {
    pub fn new(temperature: f64, humidity: f64, pressure: f64) -> Self {
        let measure_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime should be after unix epoch")
            .as_secs();

        Self {
            measure_time,
            label: None,
            temperature,
            humidity,
            pressure,
            resistance: None,
        }
    }

    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn resistance(mut self, resistance: f64) -> Self {
        self.resistance = Some(resistance);
        self
    }
}
