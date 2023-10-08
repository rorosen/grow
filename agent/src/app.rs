use crate::{
    control::{
        exhaust::{ExhaustControlArgs, ExhaustController},
        fan::{FanControlArgs, FanController},
        light::{LightControlArgs, LightController},
        pump::PumpControlArgs,
    },
    error::AppError,
    sample::water_level::{WaterLevelSampleArgs, WaterLevelSampler},
};
use clap::Parser;
use log::LevelFilter;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct App {
    #[arg(short, long, env = "RUST_LOG", default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[command(flatten)]
    exhaust_control_args: ExhaustControlArgs,

    #[command(flatten)]
    fan_control_args: FanControlArgs,

    #[command(flatten)]
    light_control_args: LightControlArgs,

    #[command(flatten)]
    pump_control_args: PumpControlArgs,

    #[command(flatten)]
    water_level_sample_args: WaterLevelSampleArgs,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        env_logger::Builder::new()
            .filter_level(self.log_level)
            .init();
        log::info!("initialized logger with log level {}", self.log_level);

        // let mut sigint = signal(SignalKind::interrupt()).map_err(AppError::SignalHandlerError)?;
        // let mut sigterm = signal(SignalKind::terminate()).map_err(AppError::SignalHandlerError)?;
        // let (finish_tx, mut finish_rx) = mpsc::channel(1);
        // let cancel_token = CancellationToken::new();

        // let _shutdown_light = LightController::start(
        //     self.light_control_args,
        //     cancel_token.clone(),
        //     finish_tx.clone(),
        // )
        // .map_err(AppError::ControlError)?;

        // let _shutdown_exhaust = ExhaustController::start(
        //     self.exhaust_control_args,
        //     cancel_token.clone(),
        //     finish_tx.clone(),
        // )
        // .map_err(AppError::ControlError)?;

        // let _shutdown_fan = FanController::start(
        //     self.fan_control_args,
        //     cancel_token.clone(),
        //     finish_tx.clone(),
        // )
        // .map_err(AppError::ControlError)?;

        // WaterLevelSampler::new(self.water_level_sample_args)
        //     .await
        //     .unwrap();

        // // drop sender so we don't wait forever later
        // drop(finish_tx);

        // tokio::select! {
        //     _ = sigint.recv() => {
        //         log::info!("shutting down on sigint");
        //     }
        //     _ = sigterm.recv() => {
        //         log::info!("shutting down on sigterm");
        //     }
        // }

        // cancel_token.cancel();
        // // wait until all tasks terminated
        // let _ = finish_rx.recv().await;

        let mut w = WaterLevelSampler::new(self.water_level_sample_args)
            .await
            .unwrap();

        w.measure_range().await.unwrap();

        Ok(())
    }
}
