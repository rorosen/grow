use crate::{
    config::air::{AirConfig, AirSensorConfig, AirSensorModel},
    control::Controller,
    datastore::DataStore,
    measure::{bme680::Bme680, AirMeasurement},
    sample::Sampler,
};
use anyhow::{Context, Result};
use futures::future::join_all;
use std::path::Path;
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, Instrument};

pub struct AirManager {
    controller: Controller,
    receiver: mpsc::Receiver<Vec<AirMeasurement>>,
    sampler: Sampler<Bme680>,
    store: DataStore,
}

impl AirManager {
    pub async fn new(
        config: &AirConfig,
        store: DataStore,
        i2c_path: &Path,
        gpio_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let controller = Controller::new(&config.control, &gpio_path)
            .context("Failed to initialize air controller")?;

        let sensors = join_all(
            config
                .sample
                .sensors
                .iter()
                .map(|(label, config)| Self::init_sensor(config, label, i2c_path)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Bme680>>>()?;

        let (sender, receiver) = mpsc::channel(8);
        let sampler = Sampler::new(config.sample.sample_rate_secs, sender, sensors)
            .context("Failed to initialize air sampler")?;

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
                            ret.context("Air manager task panicked")?
                                .context("Failed to run air manager task")?;
                        },
                        None => return Ok(()),
                    }
                }
                Some(measurements) = self.receiver.recv() => {
                    self.store
                        .add_air_measurements(measurements)
                        .await
                        .context("Failed to store air measurements")?;
                }
            }
        }
    }

    async fn init_sensor(
        config: &AirSensorConfig,
        label: &str,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Bme680> {
        match config.model {
            AirSensorModel::Bme680 => Bme680::new(i2c_path, config.address, label.to_owned())
                .await
                .with_context(|| format!("Failed to initialize {:?} air sensor", label)),
        }
    }
}
