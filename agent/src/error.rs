#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("controller error: {0}")]
    ControlError(crate::control::Error),

    #[error("failed to register signal handler: {0}")]
    SignalHandlerError(std::io::Error),
}
