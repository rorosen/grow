use std::process::ExitCode;

use axum::{http::StatusCode, routing::get, Json, Router};
use grow_measure::air::AirMeasurement;
use sqlx::SqlitePool;

mod config;
mod server;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let service = Router::new().route("/air_measurements", get(air_measurements));

    let listener = tokio::net::TcpListener::bind("::1:8000").await.unwrap();
    axum::serve(listener, service).await.unwrap();
    ExitCode::SUCCESS
}

async fn air_measurements() -> (StatusCode, Json<Vec<AirMeasurement>>) {
    let pool = SqlitePool::connect("sqlite:/tmp/grow.sqlite?mode=ro")
        .await
        .unwrap();

    let m = sqlx::query_as!(
        AirMeasurement,
        r#"
            select cast(("measure_time" / 7000) as int) * 7000 as  measure_time, label, temperature, humidity, pressure, resistance from air_measurements group by measure_time order by measure_time asc;
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    (StatusCode::OK, Json(m))
}
