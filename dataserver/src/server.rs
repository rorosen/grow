use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{self, FromRef, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use grow_measure::{
    air::AirMeasurement, light::LightMeasurement, water_level::WaterLevelMeasurement,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Debug, Clone)]
struct ServerState {
    state_dir: PathBuf,
    pools: Arc<RwLock<HashMap<String, SqlitePool>>>,
}

impl ServerState {
    fn new(state_dir: PathBuf) -> Self {
        Self {
            state_dir,
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

struct ServerSubState {
    pools: Arc<RwLock<HashMap<String, SqlitePool>>>,
}

impl FromRef<ServerState> for ServerSubState {
    fn from_ref(input: &ServerState) -> Self {
        Self {
            pools: input.pools.clone(),
        }
    }
}

struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, Deserialize)]
struct TimeParams {
    from: i64,
    to: i64,
    interval_ms: i64,
}

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new() -> Result<Self> {
        let config = Config::from_env().context("Failed to initialize config")?;

        Ok(Self { config })
    }

    pub async fn run(self) -> Result<()> {
        let state = ServerState::new(self.config.state_dir);
        let router = Router::new()
            .route("/grows", get(grows))
            .route("/:grow_id/air_measurements", get(air_measurements))
            .route("/:grow_id/light_measurements", get(light_measurements))
            .route(
                "/:grow_id/water_level_measurements",
                get(water_level_measurements),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(&self.config.listen_address)
            .await
            .context("Failed to bind TCP listener")?;
        axum::serve(listener, router)
            .await
            .context("Failed to serve")?;

        Ok(())
    }
}

async fn grows(State(state): State<ServerState>) -> Result<Json<Vec<String>>, ServerError> {
    let mut read_dir = tokio::fs::read_dir(&state.state_dir)
        .await
        .with_context(|| format!("Failed to get directory entries of {:?}", state.state_dir))?;
    let mut pools = state.pools.write().await;

    while let Some(entry) = &read_dir
        .next_entry()
        .await
        .with_context(|| format!("Failed to get directory entry of {:?}", state.state_dir))?
    {
        const SQLITE_ENDING: &str = "sqlite3";

        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|n| anyhow!("Failed to get valid unicode string from {n:?}"))?;
        let file_name = Path::new(&file_name);
        if !file_name.extension().is_some_and(|e| e == SQLITE_ENDING) {
            continue;
        }

        let grow_id = Path::new(&file_name)
            .file_stem()
            .with_context(|| format!("Failed to get grow ID from {file_name:?}"))?
            .to_str()
            .with_context(|| format!("Failed to get grow ID from {file_name:?}"))?
            .to_owned();

        if pools.contains_key(&grow_id) {
            continue;
        }

        let path = entry.path();
        let path = path
            .to_str()
            .with_context(|| format!("Failed to get valid unicode string from path {path:?}"))?;
        let url = format!("sqlite://{path}?mode=ro");
        let pool = SqlitePool::connect_lazy(&url)
            .with_context(|| format!("Failed to create connection pool with {url}"))?;

        pools.insert(grow_id, pool);
    }

    let grow_ids = pools.keys().map(|id| id.to_string()).collect();
    Ok(Json(grow_ids))
}

async fn air_measurements(
    State(state): State<ServerSubState>,
    extract::Path(grow_id): extract::Path<String>,
    time_params: Query<TimeParams>,
) -> Result<Json<Vec<AirMeasurement>>, ServerError> {
    let pools = state.pools.read().await;
    let pool = pools
        .get(&grow_id)
        .with_context(|| format!("Unknown grow ID {grow_id:?}"))?;
    let interval = time_params.interval_ms / 1000;

    let measurements = sqlx::query_as::<_, AirMeasurement>(
        r#"
        SELECT cast(("measure_time" / $1) as int) * $1 AS time,
        measure_time,
        label,
        temperature,
        humidity,
        pressure,
        resistance FROM air_measurements
        WHERE measure_time BETWEEN $2 AND $3
        GROUP BY time
        ORDER BY measure_time ASC;
    "#,
    )
    .bind(interval)
    .bind(time_params.from)
    .bind(time_params.to)
    .fetch_all(pool)
    .await
    .context("Failed to query air measurements")?;

    Ok(Json(measurements))
}

async fn light_measurements(
    State(state): State<ServerSubState>,
    extract::Path(grow_id): extract::Path<String>,
    time_params: Query<TimeParams>,
) -> Result<Json<Vec<LightMeasurement>>, ServerError> {
    let pools = state.pools.read().await;
    let pool = pools
        .get(&grow_id)
        .with_context(|| format!("Unknown grow ID {grow_id:?}"))?;
    let interval = time_params.interval_ms / 1000;

    let measurements = sqlx::query_as::<_, LightMeasurement>(
        r#"
        SELECT cast(("measure_time" / $1) as int) * $1 AS time,
        measure_time,
        label,
        illuminance FROM light_measurements
        WHERE measure_time BETWEEN $2 AND $3
        GROUP BY time
        ORDER BY measure_time ASC;
    "#,
    )
    .bind(interval)
    .bind(time_params.from)
    .bind(time_params.to)
    .fetch_all(pool)
    .await
    .context("Failed to query light measurements")?;

    Ok(Json(measurements))
}

async fn water_level_measurements(
    State(state): State<ServerSubState>,
    extract::Path(grow_id): extract::Path<String>,
    time_params: Query<TimeParams>,
) -> Result<Json<Vec<WaterLevelMeasurement>>, ServerError> {
    let pools = state.pools.read().await;
    let pool = pools
        .get(&grow_id)
        .with_context(|| format!("Unknown grow ID {grow_id:?}"))?;
    let interval = time_params.interval_ms / 1000;

    let measurements = sqlx::query_as::<_, WaterLevelMeasurement>(
        r#"
        SELECT cast(("measure_time" / $1) as int) * $1 AS time,
        measure_time,
        label,
        distance FROM water_level_measurements
        WHERE measure_time BETWEEN $2 AND $3
        GROUP BY time
        ORDER BY measure_time ASC;
    "#,
    )
    .bind(interval)
    .bind(time_params.from)
    .bind(time_params.to)
    .fetch_all(pool)
    .await
    .context("Failed to query water level measurements")?;

    Ok(Json(measurements))
}
