use std::str::FromStr;

use anyhow::{Context, Result};
use grow_measure::{
    air::AirMeasurement, light::LightMeasurement, water_level::WaterLevelMeasurement,
};
use sqlx::{sqlite::SqliteConnectOptions, QueryBuilder, Sqlite, SqlitePool};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct DataStore {
    pool: SqlitePool,
}

impl DataStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        println!("{db_url}");
        let options = SqliteConnectOptions::from_str(db_url)
            .context("foo")?
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await.context("")?;

        MIGRATOR
            .run(&pool)
            .await
            .context("Failed to run database migration")?;

        Ok(Self { pool })
    }

    pub async fn add_air_measurements(&self, measurements: Vec<AirMeasurement>) -> Result<()> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO air_measurements(measure_time, label, temperature, humidity, pressure, resistance) ",
        );
        query_builder.push_values(measurements.into_iter(), |mut b, m| {
            b.push_bind(m.measure_time)
                .push_bind(m.label)
                .push_bind(m.temperature)
                .push_bind(m.humidity)
                .push_bind(m.pressure)
                .push_bind(m.resistance);
        });
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .context("Failed to store air measurements")?;

        Ok(())
    }

    pub async fn add_light_measurements(&self, measurements: Vec<LightMeasurement>) -> Result<()> {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO light_measurements(measure_time, label, illuminance) ");
        query_builder.push_values(measurements.into_iter(), |mut b, m| {
            b.push_bind(m.measure_time)
                .push_bind(m.label)
                .push_bind(m.illuminance);
        });
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .context("Failed to store light measurements")?;

        Ok(())
    }

    pub async fn add_water_level_measurements(
        &self,
        measurements: Vec<WaterLevelMeasurement>,
    ) -> Result<()> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO water_level_measurements(measure_time, label, distance) ",
        );
        query_builder.push_values(measurements.into_iter(), |mut b, m| {
            b.push_bind(m.measure_time)
                .push_bind(m.label)
                .push_bind(m.distance);
        });
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .context("Failed to store water level measurements")?;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use chrono::Utc;
//     use sqlx::{pool::PoolOptions, sqlite::SqliteConnectOptions, ConnectOptions, Row};
//
//     #[sqlx::test]
//     async fn store_air_measurement_ok(
//         opts: PoolOptions<Sqlite>,
//         copts: SqliteConnectOptions,
//     ) -> sqlx::Result<()> {
//         let db_url = copts.get_filename().as_os_str().to_str().unwrap();
//         let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
//         let migrations_dir = std::path::Path::new(&crate_dir).join("./migrations");
//         let store = DataStore::new(&db_url, &migrations_dir).await.unwrap();
//
//         let measure_time = Utc::now();
//         let measurements = vec![AirMeasurement {
//             measure_time,
//             label: None,
//             temperature: 21.,
//             humidity: 56.,
//             pressure: 1021.,
//             resistance: None,
//         }];
//
//         store.add_air_measurements(measurements).await.unwrap();
//         let mut pool = copts.connect().await?;
//         let m = sqlx::query("select * from air_measurements")
//             .fetch_one(&store.pool)
//             .await
//             .unwrap();
//         println!("{}", m.is_empty());
//         println!("{}", m.len());
//         let t = m.get::<f64, _>("temperature");
//         println!("{t}");
//         let t = m.get::<String, _>("measure_time");
//         println!("{t}");
//         // println!("{m:?}");
//
//         Ok(())
//     }
// }
