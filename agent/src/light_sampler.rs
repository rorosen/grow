use std::{path::Path, time::Duration};

use anyhow::{bail, Context, Result};
use futures::future::join_all;
use tokio::time::{interval, Interval};
use tokio_util::sync::CancellationToken;

use crate::{
    config::light::{LightSampleConfig, LightSensorConfig, LightSensorModel},
    datastore::DataStore,
    measure::bh1750fvi::Bh1750Fvi,
    sample::Sampler,
};

pub struct LightSampler {
    interval: Interval,
    sampler: Sampler<Bh1750Fvi>,
    store: DataStore,
}

impl LightSampler {
    pub async fn new(
        config: &LightSampleConfig,
        i2c_path: &Path,
        store: DataStore,
    ) -> Result<Self> {
        let period = Duration::from_secs(config.sample_rate_secs);
        if period.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        let sensors = join_all(
            config
                .sensors
                .iter()
                .map(|(label, config)| Self::init_sensor(config, label, i2c_path)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<Bh1750Fvi>>>()?;

        Ok(Self {
            interval: interval(period),
            sampler: Sampler::new(sensors),
            store,
        })
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    let measurements = self
                        .sampler
                        .take_measurements(cancel_token.clone())
                        .await
                        .context("Failed to take light measurements")?;

                    self.store
                        .add_light_measurements(measurements)
                        .await
                        .context("Failed to store light measurements")?;
                }
                _ = cancel_token.cancelled() => {
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
