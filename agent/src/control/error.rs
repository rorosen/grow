#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize a new gpio instance: {0}")]
    InitGpioFailed(rppal::gpio::Error),

    #[error("failed to get gpio pin: {0}")]
    GetPinFailed(rppal::gpio::Error),

    #[error("invalid {0} controller arguments: {1}")]
    InvalidArgs(String, String),
}
