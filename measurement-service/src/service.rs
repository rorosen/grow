use diesel::Connection;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use grow_utils::api::grow::{
    measurement_service_server::{MeasurementService, MeasurementServiceServer},
    AirMeasurement, BatchCreateAirMeasurementsRequest, BatchCreateLightMeasurementsRequest,
    BatchCreateWaterLevelMeasurementsRequest, LightMeasurement, WaterLevelMeasurement,
};
use tonic::{transport::Server, Request, Response, Status};

use crate::{error::AppError, models::StorageAirMeasurement, schema};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct Service {
    pool: Pool<AsyncPgConnection>,
}

impl Service {
    pub async fn new(postgres_uri: String) -> Result<Self, AppError> {
        run_migrations(postgres_uri.clone(), MIGRATIONS).await?;

        let pool_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(postgres_uri);
        let pool = Pool::builder(pool_config)
            .build()
            .map_err(AppError::BuildPoolFailed)?;

        Ok(Self { pool })
    }

    pub async fn run(self, address: String, port: u16) -> Result<(), AppError> {
        let addr = format!("{}:{}", address, port)
            .parse()
            .map_err(AppError::AddrParse)?;

        Server::builder()
            .add_service(MeasurementServiceServer::new(self))
            .serve(addr)
            .await
            .map_err(AppError::ServerError)
    }
}

#[tonic::async_trait]
impl MeasurementService for Service {
    async fn create_air_measurement(
        &self,
        request: Request<AirMeasurement>,
    ) -> Result<Response<()>, Status> {
        let m = StorageAirMeasurement::try_from(request.into_inner()).map_err(|e| {
            log::error!("failed to convert measurement: {e}");
            Status::invalid_argument("failed to convert measurement")
        })?;

        let mut conn = self.pool.get().await.map_err(|e| {
            log::error!("failed to get postgresql connection: {e}");
            Status::unavailable("failed to get postgresql connection")
        })?;

        diesel::insert_into(schema::air_measurements::table)
            .values(&m)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("failed to insert measurement: {e}");
                Status::unavailable("failed to insert measurement")
            })?;

        Ok(Response::new(()))
    }

    async fn batch_create_air_measurements(
        &self,
        request: Request<BatchCreateAirMeasurementsRequest>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn create_light_measurement(
        &self,
        request: Request<LightMeasurement>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn batch_create_light_measurements(
        &self,
        request: Request<BatchCreateLightMeasurementsRequest>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn create_water_level_measurement(
        &self,
        request: Request<WaterLevelMeasurement>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }

    async fn batch_create_water_level_measurements(
        &self,
        request: Request<BatchCreateWaterLevelMeasurementsRequest>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

async fn run_migrations(
    postgres_uri: String,
    migrations: EmbeddedMigrations,
) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&postgres_uri)
            .map_err(|e| AppError::MigrationFailed(format!("could not connect: {e}")))?;

        match conn.run_pending_migrations(migrations) {
            Ok(versions) => {
                versions
                    .iter()
                    .for_each(|v| log::info!("run embedded migration \"{v}\" successfully"));

                Ok(())
            }
            Err(err) => Err(AppError::MigrationFailed(err.to_string())),
        }
    })
    .await
    .unwrap()
}
