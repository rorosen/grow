use std::process::{ExitCode, Termination};

use clap::Parser;
use grow_agent::{app::App, error::AppError};

pub enum Exit {
    Ok,
    Err(AppError),
}

impl Termination for Exit {
    fn report(self) -> std::process::ExitCode {
        match self {
            Exit::Ok => ExitCode::from(0),
            Exit::Err(err) => {
                log::error!("failed to run application: {}", err);
                ExitCode::from(1)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Exit {
    let args = App::parse();

    env_logger::Builder::new()
        .filter_level(args.log_level)
        .init();
    log::info!("initialized logger with log level {}", args.log_level);

    match args.run().await {
        Ok(_) => Exit::Ok,
        Err(err) => Exit::Err(err),
    }
}
