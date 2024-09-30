use std::path::Path;

use crate::{
    config::water_level::{WaterLevelConfig, WaterLevelSensorConfig, WaterLevelSensorModel},
    control::Controller,
    datastore::DataStore,
    measure::{vl53l0x::Vl53L0X, WaterLevelMeasurement},
    sample::Sampler,
};

use anyhow::{Context, Result};
use futures::future::join_all;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, Instrument};

pub struct WaterLevelManager {
    controller: Controller,
    receiver: mpsc::Receiver<Vec<WaterLevelMeasurement>>,
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

        let (sender, receiver) = mpsc::channel(8);
        let sampler = Sampler::new(config.sample.sample_rate_secs, sender, sensors)
            .context("Failed to initialize water level sampler")?;

        Ok(Self {
            controller,
            receiver,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut set = JoinSet::new();
        set.spawn(
            self.controller
                .run(cancel_token.clone())
                .instrument(debug_span!("controller")),
        );
        set.spawn(
            self.sampler
                .run(cancel_token.clone())
                .instrument(debug_span!("sampler")),
        );

        loop {
            tokio::select! {
                res = set.join_next() => {
                    match res {
                        Some(ret) => {
                            ret.context("Water level task panicked")?
                                .context("Failed to run water level task")?;
                        },
                        None => return Ok(()),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    self.store
                        .add_water_level_measurements(measurements)
                        .await
                        .context("Failed to store water level measurements")?;
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
