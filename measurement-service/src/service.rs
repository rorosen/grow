use grow_utils::{
    api::grow::{
        measurement_service_server::{MeasurementService, MeasurementServiceServer},
        AirMeasurement, LightMeasurement, WaterLevelMeasurement,
    },
    StorageAirMeasurement, StorageLightMeasurement, StorageWaterLevelMeasurement,
};
use mongodb::{Collection, Database};
use tonic::{transport::Server, Request, Response, Status};

use crate::{
    app::{
        AIR_MEASUREMENTS_COLLECTION, LIGHT_MEASUREMENTS_COLLECTION,
        WATER_LEVEL_MEASUREMENTS_COLLECTION,
    },
    error::AppError,
};

pub struct Service {
    air_measurements: Collection<StorageAirMeasurement>,
    light_measurements: Collection<StorageLightMeasurement>,
    water_level_measurements: Collection<StorageWaterLevelMeasurement>,
}

impl Service {
    pub async fn new(db: &Database) -> Result<Self, AppError> {
        Ok(Self {
            air_measurements: db.collection::<StorageAirMeasurement>(AIR_MEASUREMENTS_COLLECTION),
            light_measurements: db
                .collection::<StorageLightMeasurement>(LIGHT_MEASUREMENTS_COLLECTION),
            water_level_measurements: db
                .collection::<StorageWaterLevelMeasurement>(WATER_LEVEL_MEASUREMENTS_COLLECTION),
        })
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
