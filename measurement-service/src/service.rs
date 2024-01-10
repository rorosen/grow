use api::gen::grow::{
    measurement_service_server::MeasurementService, AirMeasurement, AirMeasurements,
    LightMeasurements,
};
use bson::DateTime;
use clap::Parser;
use mongodb::{
    options::{CreateCollectionOptions, TimeseriesGranularity, TimeseriesOptions},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

use crate::error::AppError;

const AIR_MEASUREMENTS_COLLECTION: &str = "air_measurements";
const LIGHT_MEASUREMENTS_COLLECTION: &str = "light_measurements";
const TIME_FIELD: &str = "measure_time";

#[derive(Debug, Parser)]
pub struct ServiceArgs {
    /// The MongoDB connection string
    #[arg(
        long,
        env = "GROW_MONGODB_URI",
        default_value_t = String::from("mongodb://localhost:27017/measurements")
    )]
    mongodb_uri: String,

    /// The database to use
    #[arg(long, env = "GROW_DATABASE")]
    database: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AirM {
    measure_time: DateTime,
    left: Option<AirMeasurement>,
    right: Option<AirMeasurement>,
}

impl From<AirMeasurements> for AirM {
    fn from(value: AirMeasurements) -> Self {
        AirM {
            measure_time: DateTime::from_millis(value.measure_time),
            left: value.left,
            right: value.right,
        }
    }
}

pub struct Service {
    air_measurements: Collection<AirM>,
    light_measurements: Collection<LightMeasurements>,
}

impl Service {
    pub async fn new(args: ServiceArgs) -> Result<Self, AppError> {
        let client = Client::with_uri_str(&args.mongodb_uri)
            .await
            .map_err(AppError::GetMongoClient)?;

        let db = client.database(&args.database);
        let ts_options = CreateCollectionOptions::builder()
            .timeseries(
                TimeseriesOptions::builder()
                    .time_field(TIME_FIELD.into())
                    .granularity(Some(TimeseriesGranularity::Minutes))
                    .build(),
            )
            .build();

        db.create_collection(AIR_MEASUREMENTS_COLLECTION, ts_options.clone())
            .await
            .map_err(AppError::CreateCollection)?;

        db.create_collection(LIGHT_MEASUREMENTS_COLLECTION, ts_options)
            .await
            .map_err(AppError::CreateCollection)?;

        Ok(Service {
            air_measurements: db.collection::<AirM>(AIR_MEASUREMENTS_COLLECTION),
            light_measurements: db.collection::<LightMeasurements>(LIGHT_MEASUREMENTS_COLLECTION),
        })
    }
}

#[tonic::async_trait]
impl MeasurementService for Service {
    async fn create_air_measurements(
        &self,
        request: Request<AirMeasurements>,
    ) -> Result<Response<()>, Status> {
        if let Err(err) = self
            .air_measurements
            .insert_one(AirM::from(request.into_inner()), None)
            .await
        {
            let msg = format!("failed to insert air measurement: {err}");
            log::error!("{msg}");
            return Err(Status::unavailable(msg));
        }

        Ok(Response::new(()))
    }

    async fn create_light_measurements(
        &self,
        request: Request<LightMeasurements>,
    ) -> Result<Response<()>, Status> {
        if let Err(err) = self
            .light_measurements
            .insert_one(request.into_inner(), None)
            .await
        {
            let msg = format!("failed to insert light measurement: {err}");
            log::error!("{msg}");
            return Err(Status::unavailable(msg));
        }

        Ok(Response::new(()))
    }
}
