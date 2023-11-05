#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("controller error: {0}")]
    ControlError(super::control::Error),
}
