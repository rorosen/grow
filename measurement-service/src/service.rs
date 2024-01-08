use api::gen::grow::{
    measurement_service_server::MeasurementService, AirMeasurement, LightMeasurement,
};
use tonic::{Request, Response, Status};

use crate::error::AppError;

pub struct Service {}

impl Service {
    pub async fn new() -> Result<Self, AppError> {
        Ok(Service {})
    }
}

#[tonic::async_trait]
impl MeasurementService for Service {
    async fn create_air_measurement(
        &self,
        request: Request<AirMeasurement>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented(""))
    }

    async fn create_light_measurement(
        &self,
        request: Request<LightMeasurement>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("message"))
    }
}
