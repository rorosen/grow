use std::process::ExitCode;

use clap::Parser;
use grow_agent::app::App;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let app = App::parse();

    match app.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("failed to run: {}", err);
            ExitCode::FAILURE
        }
    }
}
