use api::gen::grow::measurement_service_server::MeasurementServiceServer;
use clap::Parser;
use tonic::transport::Server;

use crate::{
    error::AppError,
    service::{Service, ServiceArgs},
};

#[derive(Debug, Parser)]
pub struct App {
    /// The address to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_ADDRESS",
        default_value_t = String::from("[::1]")
    )]
    listen_address: String,

    /// The port to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_PORT",
        default_value_t = 10001
    )]
    listen_port: u16,

    #[command(flatten)]
    service_args: ServiceArgs,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        let addr = format!("{}:{}", self.listen_address, self.listen_port)
            .parse()
            .map_err(AppError::AddrParse)?;

        let svc = Service::new(self.service_args).await?;

        Server::builder()
            .add_service(MeasurementServiceServer::new(svc))
            .serve(addr)
            .await
            .map_err(AppError::ServerError)
    }
}
