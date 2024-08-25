use std::{env, process::ExitCode};

use agent::Agent;

mod agent;
mod air;
mod config;
mod control;
mod light;
mod sample;
mod water_level;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let config_path = env::var("GROW_AGENT_CONFIG_PATH")
        .expect("Environment variable GROW_AGENT_CONFIG_PATH must be set to a valid value");

    let agent = match Agent::new(&config_path) {
        Ok(agent) => agent,
        Err(err) => {
            log::error!("Failed to initialize from config at {config_path:?}: {err:#}");
            return ExitCode::FAILURE;
        }
    };

    match agent.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("Failed to run agent: {err:#}");
            ExitCode::FAILURE
        }
    }
}
