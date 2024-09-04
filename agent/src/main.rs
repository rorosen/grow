use std::{env, process::ExitCode};

use agent::Agent;

mod agent;
mod air;
mod config;
mod control;
mod datastore;
mod light;
mod sample;
mod water_level;

const PRINT_DEFAULT_CONFIG: &str = "--print-default-config";

#[tokio::main]
async fn main() -> ExitCode {
    match env::args().nth(1) {
        Some(arg) if arg == PRINT_DEFAULT_CONFIG => {
            let stdout = std::io::stdout().lock();
            if let Err(err) = serde_json::to_writer_pretty(stdout, &config::Config::default()) {
                eprintln!("Failed to print default config: {err:#?}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(arg) => {
            eprintln!("Unknown argument {arg:?}");
            eprintln!("Pass {PRINT_DEFAULT_CONFIG:?} to print the default configuration and exit");
            eprintln!("Or pass no arguments to start the agent in daemon mode");
            ExitCode::FAILURE
        }
        None => {
            env_logger::init();

            let agent = match Agent::new().await {
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
    }
}
