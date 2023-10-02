use std::process::ExitCode;

use clap::Parser;
use grow_agent::app::App;

#[tokio::main]
async fn main() -> ExitCode {
    let app = App::parse();

    match app.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("failed to run app: {}", err);
            ExitCode::FAILURE
        }
    }
}
