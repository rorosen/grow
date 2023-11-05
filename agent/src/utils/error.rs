#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open {file:?}: {err}")]
    OpenError {
        file: &'static str,
        err: tokio::io::Error,
    },

    #[error("failed to set i2c slave address through ioctl system call")]
    I2cSlaveAddrError,

    #[error("failed to write to i2c: {0}")]
    I2cWriteError(tokio::io::Error),

    #[error("failed to read from i2c: {0}")]
    I2cReadError(tokio::io::Error),
}
