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
