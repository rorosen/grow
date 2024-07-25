use grow_hardware::I2cError;
use serde::{Deserialize, Serialize};

pub mod air;
pub mod light;
pub mod water_level;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sensor transport error: {0}")]
    Transport(#[from] I2cError),

    #[error("failed to identify {0} sensor")]
    IdentifyFailed(String),

    #[error("{0} sensor is not initialized")]
    NotInit(&'static str),

    #[error("measurement cancelled")]
    Cancelled,
}

/// All attributes of a single air measurement.
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
    pub resistance: f64,
}

/// The distance of a single water level measurement in mm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterLevelMeasurement(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The illuminance of a single light measurement in lux.
pub struct LightMeasurement(pub f64);
