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
        let options = SqliteConnectOptions::from_str(db_url)
            .context("Failed to initialize data store connection options")?
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options)
            .await
            .context("Failed to create data store connection pool")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[sqlx::test]
    async fn add_air_measurement_ok() {
        let store = DataStore::new("sqlite::memory:").await.unwrap();
        let measure_time = Utc::now().timestamp();
        let measurements = vec![
            AirMeasurement {
                measure_time,
                label: "test".into(),
                temperature: Some(21.),
                humidity: Some(56.123),
                pressure: Some(1021.),
                resistance: None,
            },
            AirMeasurement {
                measure_time: measure_time + 100,
                label: "test".into(),
                temperature: Some(69.),
                humidity: None,
                pressure: Some(666.777),
                resistance: None,
            },
        ];

        store
            .add_air_measurements(measurements.clone())
            .await
            .unwrap();
        let retrieved_measurements =
            sqlx::query_as::<_, AirMeasurement>("SELECT * FROM air_measurements")
                .fetch_all(&store.pool)
                .await
                .unwrap();

        assert_eq!(measurements, retrieved_measurements);
    }

    #[sqlx::test]
    async fn add_light_measurement_ok() {
        let store = DataStore::new("sqlite::memory:").await.unwrap();
        let measure_time = Utc::now().timestamp();
        let measurements = vec![
            LightMeasurement {
                measure_time,
                label: "test".into(),
                illuminance: Some(123.123),
            },
            LightMeasurement {
                measure_time,
                label: "another_test".into(),
                illuminance: Some(12.34),
            },
        ];

        store
            .add_light_measurements(measurements.clone())
            .await
            .unwrap();
        let retrieved_measurements =
            sqlx::query_as::<_, LightMeasurement>("SELECT * FROM light_measurements")
                .fetch_all(&store.pool)
                .await
                .unwrap();

        assert_eq!(measurements, retrieved_measurements);
    }

    #[sqlx::test]
    async fn add_water_level_measurement_ok() {
        let store = DataStore::new("sqlite::memory:").await.unwrap();
        let measure_time = Utc::now().timestamp();
        let measurements = vec![
            WaterLevelMeasurement {
                measure_time,
                label: "foo".into(),
                distance: Some(987),
            },
            WaterLevelMeasurement {
                measure_time,
                label: "bar".into(),
                distance: Some(923),
            },
        ];

        store
            .add_water_level_measurements(measurements.clone())
            .await
            .unwrap();
        let retrieved_measurements =
            sqlx::query_as::<_, WaterLevelMeasurement>("SELECT * FROM water_level_measurements")
                .fetch_all(&store.pool)
                .await
                .unwrap();

        assert_eq!(measurements, retrieved_measurements);
    }
}
