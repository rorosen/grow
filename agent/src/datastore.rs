use std::path::Path;

use anyhow::{Context, Result};
use grow_measure::{
    air::AirMeasurement, light::LightMeasurement, water_level::WaterLevelMeasurement,
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

pub struct DataStore {
    db: SqlitePool,
}

impl DataStore {
    pub async fn new(db_url: &str, migration_dir: &Path) -> Result<Self> {
        let db = SqlitePool::connect(db_url)
            .await
            .context("Failed to create connection pool")?;
        sqlx::migrate::Migrator::new(migration_dir)
            .await
            .context("Failed to resolve migrations")?
            .run(&db)
            .await
            .context("Failed to run database migration")?;

        Ok(Self { db })
    }

    pub async fn insert_air_measurements(&self, measurements: Vec<AirMeasurement>) -> Result<()> {
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
            .execute(&self.db)
            .await
            .context("Failed to store air measurements")?;

        Ok(())
    }

    pub async fn insert_light_measurements(
        &self,
        measurements: Vec<LightMeasurement>,
    ) -> Result<()> {
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO light_measurements(measure_time, label, illuminance) ");
        query_builder.push_values(measurements.into_iter(), |mut b, m| {
            b.push_bind(m.measure_time)
                .push_bind(m.label)
                .push_bind(m.illuminance);
        });
        query_builder
            .build()
            .execute(&self.db)
            .await
            .context("Failed to store light measurements")?;

        Ok(())
    }

    pub async fn insert_water_level_measurements(
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
            .execute(&self.db)
            .await
            .context("Failed to store water level measurements")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn store_air_measurement_ok() {
        let file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", file.path().as_os_str().to_str().unwrap());
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations_dir = std::path::Path::new(&crate_dir).join("./migrations");
        let mut store = DataStore::new(&db_url, &migrations_dir).await.unwrap();

        //     let measurements = vec![AirMeasurement {
        //         measure_time: ,
        //         label: todo!(),
        //         temperature: todo!(),
        //         humidity: todo!(),
        //         pressure: todo!(),
        //         resistance: todo!(),
        //     }];
    }
}
