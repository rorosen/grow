use std::process::ExitCode;

use agent::Agent;

mod agent;
mod air;
mod config;
mod control;
mod datastore;
mod light;
mod sample;
mod water_level;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let agent = match Agent::new() {
        Ok(agent) => agent,
        Err(err) => {
            log::error!("Failed to initialize agent: {err:#}");
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
