pub mod i2c;

#[derive(Debug, thiserror::Error)]
pub enum I2cError {
    #[error("failed to open I2C bus at {file:?}: {err}")]
    Open {
        file: &'static str,
        err: tokio::io::Error,
    },

    #[error("failed to set I2C slave address \"{0:02x}\" via ioctl")]
    SlaveAddr(u8),

    #[error("failed to write to I2C: {0}")]
    Write(tokio::io::Error),

    #[error("failed to read from I2C: {0}")]
    Read(tokio::io::Error),
}
