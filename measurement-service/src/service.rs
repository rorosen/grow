use clap::Parser;
use grow_utils::{
    api::grow::{
        measurement_service_server::MeasurementService, AirMeasurement, LightMeasurement,
        WaterLevelMeasurement,
    },
    StorageAirMeasurement, StorageLightMeasurement, StorageWaterLevelMeasurement,
};
use mongodb::{
    options::{CreateCollectionOptions, TimeseriesGranularity, TimeseriesOptions},
    Client, Collection, Database,
};
use tonic::{Request, Response, Status};

use crate::error::AppError;

const AIR_MEASUREMENTS_COLLECTION: &str = "air_measurements";
const LIGHT_MEASUREMENTS_COLLECTION: &str = "light_measurements";
const WATER_LEVEL_MEASUREMENTS_COLLECTION: &str = "water_level_measurements";
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

pub struct Service {
    air_measurements: Collection<StorageAirMeasurement>,
    light_measurements: Collection<StorageLightMeasurement>,
    water_level_measurements: Collection<StorageWaterLevelMeasurement>,
}

impl Service {
    pub async fn new(args: ServiceArgs) -> Result<Self, AppError> {
        let client = Client::with_uri_str(&args.mongodb_uri)
            .await
            .map_err(AppError::GetMongoClient)?;

        let db = client.database(&args.database);
        let collection_names = db
            .list_collection_names(None)
            .await
            .map_err(AppError::ListCollection)?;

        if !collection_names
            .iter()
            .any(|c| c == AIR_MEASUREMENTS_COLLECTION)
        {
            Service::create_timeseries(&db, AIR_MEASUREMENTS_COLLECTION).await?;
        }

        if !collection_names
            .iter()
            .any(|c| c == LIGHT_MEASUREMENTS_COLLECTION)
        {
            Service::create_timeseries(&db, LIGHT_MEASUREMENTS_COLLECTION).await?;
        }

        if !collection_names
            .iter()
            .any(|c| c == WATER_LEVEL_MEASUREMENTS_COLLECTION)
        {
            Service::create_timeseries(&db, WATER_LEVEL_MEASUREMENTS_COLLECTION).await?;
        }

        Ok(Service {
            air_measurements: db.collection::<StorageAirMeasurement>(AIR_MEASUREMENTS_COLLECTION),
            light_measurements: db
                .collection::<StorageLightMeasurement>(LIGHT_MEASUREMENTS_COLLECTION),

            water_level_measurements: db
                .collection::<StorageWaterLevelMeasurement>(WATER_LEVEL_MEASUREMENTS_COLLECTION),
        })
    }

    async fn create_timeseries(db: &Database, name: &str) -> Result<(), AppError> {
        let options = CreateCollectionOptions::builder()
            .timeseries(
                TimeseriesOptions::builder()
                    .time_field(TIME_FIELD.into())
                    .granularity(Some(TimeseriesGranularity::Minutes))
                    .build(),
            )
            .build();

        db.create_collection(name, options)
            .await
            .map_err(AppError::CreateCollection)
    }
}

#[tonic::async_trait]
impl MeasurementService for Service {
    async fn create_air_measurement(
        &self,
        request: Request<AirMeasurement>,
    ) -> Result<Response<()>, Status> {
        let measurement = match StorageAirMeasurement::try_from(request.into_inner()) {
            Ok(m) => m,
            Err(err) => {
                log::error!("could not convert air measurement to storage format: {err}");
                return Err(Status::invalid_argument("invalid measurement"));
            }
        };

        if let Err(err) = self.air_measurements.insert_one(measurement, None).await {
            let msg = format!("failed to insert air measurement: {err}");
            log::error!("{msg}");
            return Err(Status::unavailable(msg));
        }

        Ok(Response::new(()))
    }

    async fn create_light_measurement(
        &self,
        request: Request<LightMeasurement>,
    ) -> Result<Response<()>, Status> {
        let measurement = match StorageLightMeasurement::try_from(request.into_inner()) {
            Ok(m) => m,
            Err(err) => {
                log::error!("could not convert light measurement to storage format: {err}");
                return Err(Status::invalid_argument("invalid measurement"));
            }
        };

        if let Err(err) = self.light_measurements.insert_one(measurement, None).await {
            let msg = format!("failed to insert light measurement: {err}");
            log::error!("{msg}");
            return Err(Status::unavailable(msg));
        }

        // let t = "2024-01-11T21:02:27.866Z"
        //     .parse::<chrono::DateTime<chrono::Utc>>()
        //     .unwrap();
        // let t = DateTime::from_chrono(t);
        // let f = doc! {"measure_time": {"$gte": t}};
        // let mut cursor = self.air_measurements.find(f, None).await.unwrap();

        // while let Some(doc) = cursor.try_next().await.unwrap() {
        //     println!("{:?}", doc)
        // }

        Ok(Response::new(()))
    }

    async fn create_water_level_measurement(
        &self,
        request: Request<WaterLevelMeasurement>,
    ) -> Result<Response<()>, Status> {
        let measurement = match StorageWaterLevelMeasurement::try_from(request.into_inner()) {
            Ok(m) => m,
            Err(err) => {
                log::error!("could not convert water level measurement to storage format: {err}");
                return Err(Status::invalid_argument("invalid measurement"));
            }
        };

        if let Err(err) = self
            .water_level_measurements
            .insert_one(measurement, None)
            .await
        {
            let msg = format!("failed to insert water level measurement: {err}");
            log::error!("{msg}");
            return Err(Status::unavailable(msg));
        }

        Ok(Response::new(()))
    }
}
