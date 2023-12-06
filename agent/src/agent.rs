use crate::{
    error::AppError,
    manage::{
        air::{AirArgs, AirManager},
        light::{LightArgs, LightManager},
        water::{WaterArgs, WaterManager},
        AirPumpControlArgs, AirPumpController, FanControlArgs, FanController,
    },
};
use clap::Parser;
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Parser)]
pub struct Agent {
    #[command(flatten)]
    light_args: LightArgs,

    #[command(flatten)]
    water_args: WaterArgs,

    #[command(flatten)]
    fan_args: FanControlArgs,

    #[command(flatten)]
    air_args: AirArgs,

    #[command(flatten)]
    air_pump_args: AirPumpControlArgs,
}

impl Agent {
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

        let fan_controller = match FanController::new(&self.fan_args) {
            Ok(controller) => controller,
            Err(err) => {
                log::error!("failed to initialize fan controller: {err}");
                return Err(AppError::Fatal);
            }
        };

        let air_manager = match AirManager::new(&self.air_args) {
            Ok(manager) => manager,
            Err(err) => {
                log::error!("failed to initialize air manager: {err}");
                return Err(AppError::Fatal);
            }
        };

        let light_manager = match LightManager::new(&self.light_args) {
            Ok(manager) => manager,
            Err(err) => {
                log::error!("failed to initialize light manager: {err}");
                return Err(AppError::Fatal);
            }
        };

        let water_manager = match WaterManager::new(&self.water_args) {
            Ok(manager) => manager,
            Err(err) => {
                log::error!("failed to initialize water manager: {err}");
                return Err(AppError::Fatal);
            }
        };

        if let Err(err) = AirPumpController::set(&self.air_pump_args) {
            log::error!("failed to sett air pump controller: {err}");
            return Err(err);
        }

        let mut set = JoinSet::new();

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("fan controller", fan_controller.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("air manager", air_manager.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("light manager", light_manager.run(cloned_token).await) });

        let cloned_token = cancel_token.clone();
        set.spawn(async move { ("water manager", water_manager.run(cloned_token).await) });

        let res = loop {
            tokio::select! {
                _ = sigint.recv() => {
                    log::info!("shutting down on sigint");
                    cancel_token.cancel();
                }
                _ = sigterm.recv() => {
                    log::info!("shutting down on sigterm");
                    cancel_token.cancel();
                }
                res = set.join_next() => {
                    match res {
                        Some(Ok((id, Ok(_)))) => log::debug!("{id} task terminated successfully"),
                        Some(Ok((id, Err(err)))) => log::warn!("{id} task terminated with error: {err}"),
                        Some(Err(err)) => {
                            log::error!("some task panicked: {err}");
                            break Err(AppError::Fatal);
                        }
                        None => {
                            log::error!("all manager tasks terminated");
                            break Ok(());
                        }
                    }
                }
            }
        };

        res
    }
}
