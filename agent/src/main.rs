use std::process::ExitCode;

use clap::Parser;
use grow_agent::agent::Agent;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let agent = Agent::parse();
    println!("{agent:?}");

    match agent.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
