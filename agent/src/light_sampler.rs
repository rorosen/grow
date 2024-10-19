use anyhow::{bail, Context, Result};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::{
    config::sample::SampleConfig, datastore::DataStore, measure::LightMeasurement, sample::Sampler,
};

pub struct LightSampler {
    receiver: broadcast::Receiver<Vec<LightMeasurement>>,
    sampler: Sampler<LightMeasurement>,
    store: DataStore,
}

impl LightSampler {
    pub async fn new(config: &SampleConfig, store: DataStore) -> Result<Option<Self>> {
        match config {
            SampleConfig::Off => None,
            SampleConfig::Interval {
                period,
                script_path,
            } => {
                let (sender, receiver) = broadcast::channel(8);
                let sampler = Sampler::new(*period, script_path, sender)
                    .context("Failed to initialize light sampler")?;

                Some(Ok(Self {
                    receiver,
                    sampler,
                    store,
                }))
            }
        }
        .transpose()
    }

    pub async fn run(mut self, cancel_token: CancellationToken) -> Result<()> {
        let mut sampler_handle = tokio::spawn(self.sampler.run(cancel_token.clone()));

        loop {
            tokio::select! {
                res = self.receiver.recv(), if !cancel_token.is_cancelled() => {
                    match res {
                        Ok(measurements) => {
                            self.store
                                .add_light_measurements(measurements)
                                .await
                                .context("Failed to store light measurements")?;
                        },
                        Err(RecvError::Lagged(num_skipped)) => warn!("Skipping {num_skipped} measurements due to lagging"),
                        Err(RecvError::Closed) => bail!("Failed to receive measurements: {}", RecvError::Closed),
                    }
                }
                res = &mut sampler_handle => {
                    res.context("Light sampler panicked")?
                        .context("Failed to run light sampler")?;

                    return Ok(());
                }
            }
        }
    }
}
