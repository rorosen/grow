use serde::{Deserialize, Serialize};

use super::control::ControlConfig;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirPumpConfig {
    #[serde(default)]
    pub control: ControlConfig,
}
