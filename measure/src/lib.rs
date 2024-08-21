pub mod air;
mod i2c;
pub mod light;
pub mod water_level;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I2C error: {0}")]
    Transport(#[from] i2c::I2cError),

    #[error("Failed to identify sensor")]
    IdentifyFailed,

    #[error("Sensor is not initialized")]
    NotInit,

    #[error("Measurement cancelled")]
    Cancelled,
}
