#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to register signal handler: {0}")]
    SignalHandlerError(std::io::Error),

    #[error("failed to open {file:?}: {err}")]
    I2cOpenError {
        file: &'static str,
        err: tokio::io::Error,
    },

    #[error("failed to set i2c slave address through ioctl system call")]
    I2cSlaveAddrError,

    #[error("failed to write to i2c: {0}")]
    I2cWriteError(tokio::io::Error),

    #[error("failed to read from i2c: {0}")]
    I2cReadError(tokio::io::Error),

    #[error("failed to identify {0} sensor")]
    IdentifyFailed(String),

    #[error("measurement cancelled")]
    Cancelled,

    #[error("failed to initialize a new gpio instance: {0}")]
    InitGpioFailed(rppal::gpio::Error),

    #[error("failed to get gpio pin: {0}")]
    GetGpioFailed(rppal::gpio::Error),

    #[error("invalid {0} controller arguments: {1}")]
    InvalidControllerArgs(String, String),

    #[error("{name} task panicked: {err}")]
    TaskPanicked {
        name: &'static str,
        err: tokio::task::JoinError,
    },
}
