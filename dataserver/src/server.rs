use std::{
    collections::HashMap,
    ffi::OsString,
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
    interval: i64,
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
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|n| anyhow!("Failed to get valid unicode string from {n:?}"))?;
        let grow_id = Path::new(&file_name)
            .file_stem()
            .with_context(|| format!("Failed to get grow ID from {file_name}"))?
            .to_str()
            .with_context(|| format!("Failed to get grow ID from {file_name}"))?
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
) -> Result<(), ServerError> {
    log::info!("{grow_id:?}");
    // let pools = state.pools.read().await;
    // let pool = pools.get(grow_id).with_context(|| format!("Failed to get database pool for {grow_id}"))?;
    Ok(())
}

async fn light_measurements(
    State(state): State<ServerSubState>,
    extract::Path(grow_id): extract::Path<String>,
    time_params: Query<TimeParams>,
) -> Result<(), ServerError> {
    Ok(())
}

async fn water_level_measurements(
    State(state): State<ServerSubState>,
    extract::Path(grow_id): extract::Path<String>,
    time_params: Query<TimeParams>,
) -> Result<(), ServerError> {
    Ok(())
}
