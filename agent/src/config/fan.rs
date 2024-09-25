use serde::{Deserialize, Serialize};

use super::control::ControlConfig;

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct FanConfig {
    #[serde(default)]
    pub control: ControlConfig,
}
