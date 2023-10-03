#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize i2c: {0}")]
    InitI2cFailed(crate::periph::Error),

    #[error("i2c error: {0}")]
    I2cActionFailed(crate::periph::Error),
}
