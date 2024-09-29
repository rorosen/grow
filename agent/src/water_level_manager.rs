use std::{path::Path, time::Duration};

use crate::{
    config::water_level::{WaterLevelConfig, WaterLevelSensorConfig, WaterLevelSensorModel},
    control::Controller,
    datastore::DataStore,
    measure::vl53l0x::Vl53L0X,
    sample::Sampler,
};

use anyhow::{bail, Context, Result};
use futures::future::join_all;
use tokio::time::{interval, Interval};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, Instrument};

pub struct WaterLevelManager {
    controller: Controller,
    interval: Interval,
    sampler: Sampler<Vl53L0X>,
    store: DataStore,
}

impl WaterLevelManager {
    pub async fn new(
        config: &WaterLevelConfig,
        store: DataStore,
        i2c_path: &Path,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let controller = Controller::new(&config.control, &gpio_path)
            .context("Failed to initialize water level controller")?;

        let period = Duration::from_secs(config.sample.sample_rate_secs);
        if period.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        let sensors = join_all(
            config
                .sample
                .sensors
                .iter()
                .map(|(label, config)| Self::init_sensor(config, label, i2c_path)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Vl53L0X>>>()?;

        Ok(Self {
            controller,
            interval: interval(period),
            sampler: Sampler::new(sensors),
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut controller_handle = tokio::spawn(
            self.controller
                .run(cancel_token.clone())
                .instrument(debug_span!("controller")),
        );

        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    let measurements = self
                        .sampler
                        .take_measurements(cancel_token.clone())
                        .await
                        .context("Failed to take water level measurements")?;

                    self.store
                        .add_water_level_measurements(measurements)
                        .await
                        .context("Failed to store water level measurements")?;
                }
                res = &mut controller_handle => {
                    res.context("Water level controller panicked")?
                        .context("Failed to run water level controller")?;
                }
            }
        }
    }

    async fn init_sensor(
        config: &WaterLevelSensorConfig,
        label: &str,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Vl53L0X> {
        match config.model {
            WaterLevelSensorModel::Vl53L0X => {
                Vl53L0X::new(i2c_path, config.address, label.to_owned())
                    .await
                    .with_context(|| format!("Failed to initialize {:?} water level sensor", label))
            }
        }
    }
}
