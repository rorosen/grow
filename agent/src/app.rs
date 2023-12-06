use crate::{
    error::AppError,
    manage::{
        air::AirArgs, light::LightArgs, water::WaterArgs, AirPumpControlArgs, FanControlArgs,
    },
};
use clap::Parser;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct App {
    #[command(flatten)]
    light_args: LightArgs,

    #[command(flatten)]
    pump_args: WaterArgs,

    #[command(flatten)]
    fan_args: FanControlArgs,

    #[command(flatten)]
    exhaust_args: AirArgs,

    #[command(flatten)]
    air_pump_args: AirPumpControlArgs,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        let mut sigint = signal(SignalKind::interrupt()).map_err(AppError::SignalHandlerError)?;
        let mut sigterm = signal(SignalKind::terminate()).map_err(AppError::SignalHandlerError)?;
        let cancel_token = CancellationToken::new();

        // let mut air_sampler = AirSampler::new(0x76).await?;
        // let air_measurement = air_sampler.measure(cancel_token.clone()).await?;
        // println!("air: {air_measurement:?}");

        // let mut light_sampler = LightSensor::new(0x23).await?;
        // let light_measurement = light_sampler.measure(cancel_token.clone()).await?;
        // println!("light: {light_measurement:?}");

        // let mut water_sampler = WaterLevelSampler::new(0x29).await?;
        // let water_measurement = water_sampler.measure().await?;
        // println!("water: {water_measurement:?}");

        let mut set = JoinSet::new();

        // let c = cancel_token.clone();
        // set.spawn(async move { ("light", LightManager::start(self.light_args, c).await) });

        // let c = cancel_token.clone();
        // set.spawn(async move { ("pump", PumpManager::start(self.pump_args, c).await) });

        // let c = cancel_token.clone();
        // set.spawn(async move { ("circulation fan", FanManager::start(self.fan_args, c).await) });

        // let c = cancel_token.clone();
        // set.spawn(async move {
        //     (
        //         "exhaust fan",
        //         ExhaustManager::start(self.exhaust_args, c).await,
        //     )
        // });

        // set.spawn(async move { ("air pump", AirPumpManager::start(self.air_pump_args).await) });

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
                // res = set.join_next() => {
                //     match res {
                //         Some(Ok((id, Ok(_)))) => log::debug!("{id} manager task terminated successfully"),
                //         Some(Ok((id, Err(err)))) => log::warn!("{id} manager task terminated with error: {err}"),
                //         Some(Err(err)) => {
                //             log::error!("some manager task panicked: {err}");
                //             break;
                //         }
                //         None => {
                //             log::error!("all manager tasks finished unexpectedly");
                //             break;
                //         }
                //     }
                // }
            }
        }

        cancel_token.cancel();

        // while let Some(res) = set.join_next().await {
        //     match res {
        //         Ok((id, Ok(_))) => log::debug!("{id} manager task terminated successfully"),
        //         Ok((id, Err(err))) => log::warn!("{id} manager task terminated with error: {err}"),
        //         Err(err) => log::error!("some manager task panicked: {err}"),
        //     }
        // }

        Ok(())
    }
}
