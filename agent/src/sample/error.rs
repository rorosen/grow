#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize i2c: {0}")]
    InitI2cFailed(crate::periph::Error),

    #[error("failed to initialize {sensor} sensor: {src}")]
    InitSensor {
        src: crate::periph::Error,
        sensor: String,
    },

    #[error("failed to identify {0} sensor")]
    IdentifyFailed(String),
}
