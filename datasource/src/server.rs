use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::config::Config;

struct ServerState {
    pool: SqlitePool,
}

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new() -> Result<Self> {
        let config = Config::from_env().context("Failed to initialize config")?;

        Ok(Self { config })
    }

    pub async fn run() -> Result<()> {
        Ok(())
    }
}
