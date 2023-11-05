#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to register signal handler: {0}")]
    SignalHandlerError(std::io::Error),
}
