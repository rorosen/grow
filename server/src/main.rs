use std::process::ExitCode;

use server::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod server;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server = match Server::new() {
        Ok(s) => s,
        Err(err) => {
            log::error!("Failed to initialize server: {err:#}");
            return ExitCode::FAILURE;
        }
    };

    match server.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("Failed to run agent: {err:#}");
            ExitCode::FAILURE
        }
    }
}

// async fn air_measurements() -> (StatusCode, Json<Vec<AirMeasurement>>) {
//     let pool = SqlitePool::connect("sqlite:/tmp/grow.sqlite?mode=ro")
//         .await
//         .unwrap();
//
//     let m = sqlx::query_as!(
//         AirMeasurement,
//         r#"
//             select cast(("measure_time" / 7000) as int) * 7000 as  measure_time, label, temperature, humidity, pressure, resistance from air_measurements group by measure_time order by measure_time asc;
//         "#
//     )
//     .fetch_all(&pool)
//     .await
//     .unwrap();
//
//     (StatusCode::OK, Json(m))
// }
