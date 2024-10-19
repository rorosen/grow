use std::{path::PathBuf, sync::LazyLock, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use pyo3::{exceptions::PyAttributeError, prelude::*, types::PyType};
use tokio::sync::{self, broadcast};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::measure::pylib::grow_pylib;

#[rustfmt::skip]
static ADD_PYTHON_MODULES: LazyLock<()> = LazyLock::new(|| {
    pyo3::append_to_inittab!(grow_pylib)
});

pub struct Sampler<M> {
    period: Duration,
    sender: broadcast::Sender<Vec<M>>,
    script_path: PathBuf,
}

impl<M> Sampler<M>
where
    M: Send + Sync + 'static + pyo3::PyClass + std::fmt::Debug + for<'a> FromPyObject<'a>,
{
    pub fn new(
        period: Duration,
        script_path: impl Into<PathBuf>,
        sender: broadcast::Sender<Vec<M>>,
    ) -> Result<Self> {
        if period.is_zero() {
            bail!("Sample rate cannot be zero");
        }

        Ok(Self {
            sender,
            period,
            script_path: script_path.into(),
        })
    }

    pub async fn run(self, cancel_token: CancellationToken) -> Result<()> {
        tracing::debug!("Starting sampler");
        let _ = &*ADD_PYTHON_MODULES;
        pyo3::prepare_freethreaded_python();

        let (sender, receiver) = sync::mpsc::channel(8);
        let mut py_handle = tokio::task::spawn_blocking(move || {
            run_python(self.script_path, receiver, self.sender)
        });

        let mut interval = tokio::time::interval(self.period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = interval.tick(), if !cancel_token.is_cancelled() => {
                    sender
                        .send(())
                        .await
                        .context("Failed to trigger measurement")?;
                }
                _ = cancel_token.cancelled() => {
                    // drop(sender);
                    // py_handle
                    //     .await
                    //     .context("Measurement task panicked")?
                    //     .context("Measurement task returned with error")?;

                    return Ok(());
                }
                res = &mut py_handle => {
                    res.context("Measurement task panicked")?
                        .context("Measurement task returned with error")?;

                    return Ok(());
                }
            }
        }
    }
}

fn run_python<M>(
    module_path: PathBuf,
    mut receiver: tokio::sync::mpsc::Receiver<()>,
    sender: tokio::sync::broadcast::Sender<Vec<M>>,
) -> Result<()>
where
    M: Send + Sync + 'static + for<'a> FromPyObject<'a>,
{
            tracing::error!("Running init function");
    let py_app = std::fs::read_to_string(module_path)?;
    Python::with_gil(|py| -> Result<()> {
        let py_module = PyModule::from_code_bound(py, &py_app, "", "")
            .context("Failed to load python module")?;
        let py_measure: Py<PyAny> = py_module
            .getattr("measure")
            .context("Failed to get measure function")?
            .into();
        let py_init: Option<Py<PyAny>> = match py_module.getattr("init") {
            Ok(m) => Some(m.into()),
            Err(err)
                if err
                    .get_type_bound(py)
                    .is(&PyType::new_bound::<PyAttributeError>(py)) =>
            {
                None
            }
            Err(err) => bail!("Failed to get init function: {err:#}"),
        };

        if let Some(init) = py_init {
            init.call0(py).context("Failed to call init function")?;
        }

        loop {
            let res = py_measure
                .call0(py)
                .context("Failed to call measure function")?;
            let measurements: Vec<M> = res
                .extract(py)
                .context("Failed to extract measurements from python type")?;
            let should_break = py.allow_threads(|| -> Result<bool> {
                sender
                    .send(measurements)
                    .map_err(|_| anyhow!("channel closed"))
                    .context("Failed to send measurements")?;

                Ok(receiver.blocking_recv().is_none())
            })?;

            if should_break {
                break Ok(());
            }
        }
    })
    .context("Failed to run python interpreter")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::measure::{AirMeasurement, WaterLevelMeasurement};

    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn foo() {
        let (sender, mut receiver) = broadcast::channel(8);
        let cancel_token = CancellationToken::new();
        let s: Sampler<AirMeasurement> = Sampler::new(
            Duration::from_secs(1),
            concat!(env!("CARGO_MANIFEST_DIR"), "/../test.py"),
            sender,
        )
        .unwrap();
        // let s2: Sampler<WaterLevelMeasurement> = Sampler::new(
        //     Duration::from_secs(2),
        //     concat!(env!("CARGO_MANIFEST_DIR"), "/../testy.py"),
        //     sender,
        // )
        // .unwrap();

        let c = cancel_token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            c.cancel();
        });

        let h1 = tokio::spawn(s.run(cancel_token.clone()));
        // let h2 = tokio::spawn(s2.run(cancel_token.clone()));

        while let Ok(m) = receiver.recv().await {
            dbg!(m);
        }

        println!("{:?}", h1.await);
        // println!("{:?}", h2.await);
    }
}
