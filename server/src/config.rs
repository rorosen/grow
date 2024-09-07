use std::{env, path::PathBuf};

use anyhow::{Context, Result};

pub struct Config {
    pub listen_address: String,
    pub listen_port: u16,
    pub state_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let listen_address = env::var("GROW_LISTEN_ADDRESS").unwrap_or("::1".into());
        let listen_port = env::var("GROW_LISTEN_PORT").unwrap_or("8080".into());
        let listen_port = listen_port.parse().context("Failed to parse listen port")?;
        let state_dirs = env::var("STATE_DIRECTORY")
            .context("Failed to read STATE_DIRECTORY from environment")?;
        let state_dir = state_dirs
            .split(':')
            .next()
            .with_context(|| format!("Failed to get state directory from {state_dirs:?}"))?
            .into();

        Ok(Self {
            listen_address,
            listen_port,
            state_dir,
        })
    }
}
