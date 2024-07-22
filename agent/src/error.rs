use grow_measure::SensorError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to register signal handler: {0}")]
    SignalHandlerError(std::io::Error),

    #[error("sensor error")]
    SensorError(#[from] SensorError),

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

    #[error("terminating due to fatal error")]
    Fatal,
}
