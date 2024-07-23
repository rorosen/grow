use std::{env, process::ExitCode};

use agent::Agent;

mod agent;
mod config;
mod manage;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let Ok(config_path) = env::var("GROW_AGENT_CONFIG_PATH") else {
        log::error!("environment variable \"GROW_AGENT_CONFIG_PATH\" must be set to a valid value");
        return ExitCode::FAILURE;
    };

    let agent = match Agent::new(&config_path) {
        Ok(agent) => agent,
        Err(err) => {
            log::error!("failed to initialize agent from config at {config_path:?}: {err:#}");
            return ExitCode::FAILURE;
        }
    };

    match agent.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}
