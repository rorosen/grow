#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to parse socket address: {0}")]
    AddrParse(std::net::AddrParseError),

    #[error("failed to get mongodb client: {0}")]
    GetMongoClient(mongodb::error::Error),

    #[error("failed to create mongo collection: {0}")]
    CreateCollection(mongodb::error::Error),

    #[error("failed to insert {ty} measurement: {err}")]
    InsertFailed {
        ty: &'static str,
        err: mongodb::error::Error,
    },

    #[error("failed to serve: {0}")]
    ServerError(tonic::transport::Error),
}
