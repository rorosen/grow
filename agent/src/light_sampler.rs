use std::path::Path;

use anyhow::{Context, Result};
use futures::future::join_all;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    config::light::{LightSampleConfig, LightSensorConfig, LightSensorModel},
    datastore::DataStore,
    measure::{bh1750fvi::Bh1750Fvi, LightMeasurement},
    sample::Sampler,
};

pub struct LightSampler {
    receiver: mpsc::Receiver<Vec<LightMeasurement>>,
    sampler: Sampler<Bh1750Fvi>,
    store: DataStore,
}

impl LightSampler {
    pub async fn new(
        config: &LightSampleConfig,
        i2c_path: &Path,
        store: DataStore,
    ) -> Result<Self> {
        let sensors = join_all(
            config
                .sensors
                .iter()
                .map(|(label, config)| Self::init_sensor(config, label, i2c_path)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Bh1750Fvi>>>()?;

        let (sender, receiver) = mpsc::channel(8);
        let sampler = Sampler::new(config.sample_rate_secs, sender, sensors)
            .context("Failed to initialize light sampler")?;

        Ok(Self {
            receiver,
            sampler,
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut sampler_handle = tokio::spawn(self.sampler.run(cancel_token.clone()));

        loop {
            tokio::select! {
                Some(measurements) = self.receiver.recv() => {
                    self.store
                        .add_light_measurements(measurements)
                        .await
                        .context("Failed to store water level measurements")?;
                }
                res = &mut sampler_handle => {
                    res.context("Light sampler panicked")?
                        .context("Failed to run light sampler")?;

                    return Ok(());
                }
            }
        }
    }

    async fn init_sensor(
        config: &LightSensorConfig,
        label: &str,
        i2c_path: impl AsRef<Path>,
    ) -> Result<Bh1750Fvi> {
        match config.model {
            LightSensorModel::Bh1750Fvi => {
                Bh1750Fvi::new(i2c_path, config.address, label.to_owned())
                    .await
                    .with_context(|| format!("Failed to initialize {:?} light sensor", label))
            }
        }
    }
}
