use clap::Parser;
use mongodb::{
    options::{CreateCollectionOptions, TimeseriesGranularity, TimeseriesOptions},
    Client, Database,
};

use crate::{datasource::Datasource, error::AppError, service::Service};

pub(crate) const AIR_MEASUREMENTS_COLLECTION: &str = "air_measurements";
pub(crate) const LIGHT_MEASUREMENTS_COLLECTION: &str = "light_measurements";
pub(crate) const WATER_LEVEL_MEASUREMENTS_COLLECTION: &str = "water_level_measurements";
const TIME_FIELD: &str = "measure_time";

#[derive(Debug, Parser)]
pub struct App {
    /// The address for the server to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_ADDRESS",
        default_value_t = String::from("[::1]")
    )]
    listen_address: String,

    /// The port for teh server to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_SERVICE_LISTEN_PORT",
        default_value_t = 10001
    )]
    listen_port: u16,

    /// The address for the datasource to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_DATASOURCE_ADDRESS",
        default_value_t = String::from("[::1]")
    )]
    datasource_address: String,

    /// The port for the datasource to listen on
    #[arg(
        long,
        env = "GROW_MEASUREMENT_DATASOURCE_PORT",
        default_value_t = 10002
    )]
    datasource_port: u16,

    /// The MongoDB connection string
    #[arg(
        long,
        env = "GROW_MONGODB_URI",
        default_value_t = String::from("mongodb://localhost:27017/measurements")
    )]
    mongodb_uri: String,

    /// The MongoDB database to use
    #[arg(long, env = "GROW_DATABASE")]
    database: String,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        let client = Client::with_uri_str(&self.mongodb_uri)
            .await
            .map_err(AppError::GetMongoClient)?;

        let db = client.database(&self.database);
        let collection_names = db
            .list_collection_names(None)
            .await
            .map_err(AppError::ListCollection)?;

        Self::ensure_timeseries(&db, &collection_names, AIR_MEASUREMENTS_COLLECTION).await?;
        Self::ensure_timeseries(&db, &collection_names, LIGHT_MEASUREMENTS_COLLECTION).await?;
        Self::ensure_timeseries(&db, &collection_names, WATER_LEVEL_MEASUREMENTS_COLLECTION)
            .await?;

        let service = Service::new(&db).await?;
        let datasource = Datasource::new(db).await;

        let service_handle = tokio::spawn(service.run(self.listen_address, self.listen_port));
        let datasource_handle =
            tokio::spawn(datasource.run(self.datasource_address, self.datasource_port));

        tokio::select! {
            Ok(res) = service_handle => {
                return res;
            },
            Ok(res) = datasource_handle => {
                return res;
            },
        }
    }

    async fn ensure_timeseries(
        db: &Database,
        collection_names: &Vec<String>,
        name: &str,
    ) -> Result<(), AppError> {
        if !collection_names.iter().any(|c| c == name) {
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
                .map_err(AppError::CreateCollection)?;
        }

        Ok(())
    }
}
