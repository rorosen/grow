use std::process::ExitCode;

use agent::Agent;
use clap::Parser;

mod agent;
mod manage;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let agent = Agent::parse();

    match agent.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
