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

    #[error("failed to list mongo collections: {0}")]
    ListCollection(mongodb::error::Error),

    #[error("failed to run server: {0}")]
    ServerError(tonic::transport::Error),

    #[error("failed to run datasource: {0}")]
    DatasourceError(std::io::Error),

    #[error("task panicked: {0}")]
    TaskPanic(tokio::task::JoinError),
}
