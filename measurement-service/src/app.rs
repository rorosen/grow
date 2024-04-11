use clap::Parser;

use crate::{error::AppError, service::Service};

#[derive(Debug, Parser)]
pub struct App {
    /// The address for the server to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_ADDRESS",
        default_value_t = String::from("[::1]")
    )]
    listen_address: String,

    ///ort for teh server to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_PORT",
        default_value_t = 10001
    )]
    listen_port: u16,

    /// The MongoDB connection string
    #[arg(
        long,
        env = "GROW_POSTGRES_URI",
        default_value_t = String::from("postgresql://localhost:5432/grow")
    )]
    postgres_uri: String,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        let svc = Service::new(self.postgres_uri).await?;
        svc.run(self.listen_address, self.listen_port).await
    }
}
