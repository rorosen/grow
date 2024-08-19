use grow_hardware::I2cError;
use serde::{Deserialize, Serialize};

pub mod air;
pub mod light;
pub mod water_level;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sensor transport error: {0}")]
    Transport(#[from] I2cError),

    #[error("failed to identify sensor")]
    IdentifyFailed,

    #[error("sensor is not initialized")]
    NotInit,

    #[error("measurement cancelled")]
    Cancelled,
}

/// A single air measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A single water level measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterLevelMeasurement {
    /// The distance in mm.
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A single light measurement.
pub struct LightMeasurement {
    /// The illuminance in lux.
    pub illuminance: f64,
}
