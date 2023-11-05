use crate::{
    error::AppError,
    manage::{
        air_pump::{AirPumpArgs, AirPumpManager},
        exhaust::{ExhaustArgs, ExhaustManager},
        fan::{FanArgs, FanManager},
        light::{LightArgs, LightManager},
        pump::PumpArgs,
        PumpManager,
    },
};
use clap::Parser;
use log::LevelFilter;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct App {
    #[arg(short, long, env = "RUST_LOG", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[command(flatten)]
    light_args: LightArgs,

    #[command(flatten)]
    pump_args: PumpArgs,

    #[command(flatten)]
    fan_args: FanArgs,

    #[command(flatten)]
    exhaust_args: ExhaustArgs,

    #[command(flatten)]
    air_pump_args: AirPumpArgs,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        env_logger::Builder::new()
            .filter_level(self.log_level)
            .init();
        log::info!("initialized logger with log level {}", self.log_level);

        let mut sigint = signal(SignalKind::interrupt()).map_err(AppError::SignalHandlerError)?;
        let mut sigterm = signal(SignalKind::terminate()).map_err(AppError::SignalHandlerError)?;
        let cancel_token = CancellationToken::new();

        let mut set = JoinSet::new();

        let c = cancel_token.clone();
        set.spawn(async move { ("light", LightManager::start(self.light_args, c).await) });

        let c = cancel_token.clone();
        set.spawn(async move { ("pump", PumpManager::start(self.pump_args, c).await) });

        let c = cancel_token.clone();
        set.spawn(async move { ("circulation fan", FanManager::start(self.fan_args, c).await) });

        let c = cancel_token.clone();
        set.spawn(async move {
            (
                "exhaust fan",
                ExhaustManager::start(self.exhaust_args, c).await,
            )
        });

        set.spawn(async move { ("air pump", AirPumpManager::start(self.air_pump_args).await) });

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    log::info!("shutting down on sigint");
                    break;
                }
                _ = sigterm.recv() => {
                    log::info!("shutting down on sigterm");
                    break;
                }
                res = set.join_next() => {
                    match res {
                        Some(Ok((id, Ok(_)))) => log::debug!("{id} manager task terminated successfully"),
                        Some(Ok((id, Err(err)))) => log::warn!("{id} manager task terminated with error: {err}"),
                        Some(Err(err)) => {
                            log::error!("some task panicked: {err}");
                            break;
                        }
                        None => {
                            log::error!("all manager tasks finished unexpectedly");
                            break;
                        }
                    }
                }
            }
        }

        cancel_token.cancel();

        while let Some(res) = set.join_next().await {
            match res {
                Ok((id, Ok(_))) => log::debug!("{id} manager task terminated successfully"),
                Ok((id, Err(err))) => log::warn!("{id} manager task terminated with error: {err}"),
                Err(err) => log::error!("some task panicked: {err}"),
            }
        }

        Ok(())
    }
}
