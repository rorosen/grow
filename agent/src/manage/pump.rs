use crate::{control::pump::PumpController, sample::water_level::WaterLevelSampler};

use super::Error;

pub struct PumpManager {
    controller: PumpController,
    sampler: WaterLevelSampler,
}

impl PumpManager {
    pub async fn start() -> Result<(), Error> {
        Ok(())
    }
}
