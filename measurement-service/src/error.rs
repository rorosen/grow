#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to parse socket address: {0}")]
    AddrParse(std::net::AddrParseError),

    #[error("failed to run server: {0}")]
    ServerError(tonic::transport::Error),

    #[error("database migration failed: {0}")]
    MigrationFailed(String),

    #[error("failed to build postgresql connection pool: {0}")]
    BuildPoolFailed(diesel_async::pooled_connection::deadpool::BuildError),
}
