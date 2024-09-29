use crate::{
    config::air::{AirConfig, AirSensorConfig, AirSensorModel},
    control::Controller,
    datastore::DataStore,
    measure::bme680::Bme680,
    sample::Sampler,
};
use anyhow::{bail, Context, Result};
use futures::future::join_all;
use std::{path::Path, time::Duration};
use tokio::
    time::{interval, Interval}
;
use tokio_util::sync::CancellationToken;
use tracing::{debug_span, Instrument};

pub struct AirManager {
    controller: Controller,
    interval: Interval,
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
        .collect::<Result<Vec<Bme680>>>()?;

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
                        .context("Failed to take air measurements")?;

                    self.store
                        .add_air_measurements(measurements)
                        .await
                        .context("Failed to store air measurements")?;
                }
                res = &mut controller_handle => {
                    res.context("Air controller panicked")?
                        .context("Failed to run air controller")?;
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
