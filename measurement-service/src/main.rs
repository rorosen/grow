use std::process::ExitCode;

use env_logger::Env;
use measurement_service::app::App;

use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let app = App::parse();

    match app.run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}
