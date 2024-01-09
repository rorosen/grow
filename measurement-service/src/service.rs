use api::gen::grow::{
    measurement_service_server::MeasurementService, AirMeasurements, LightMeasurements,
};
use clap::Parser;
use mongodb::Client;
use tonic::{Request, Response, Status};

use crate::error::AppError;

#[derive(Debug, Parser)]
pub struct ServiceArgs {
    /// The MongoDB connection string
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_MONGODB_URI",
        default_value_t = String::from("mongodb://localhost:27017/measurements")
    )]
    mongodb_uri: String,
}

pub struct Service {
    client: Client,
}

impl Service {
    pub async fn new(args: ServiceArgs) -> Result<Self, AppError> {
        let client = Client::with_uri_str(&args.mongodb_uri)
            .await
            .map_err(AppError::GetMongoClient)?;

        Ok(Service { client })
    }
}

#[tonic::async_trait]
impl MeasurementService for Service {
    async fn create_air_measurements(
        &self,
        request: Request<AirMeasurements>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn create_light_measurements(
        &self,
        request: Request<LightMeasurements>,
    ) -> Result<Response<()>, Status> {
        log::info!("got a request: {:?}", request.into_inner());

        Ok(Response::new(()))
    }
}
