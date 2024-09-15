use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AirPumpConfig {
    #[serde(default)]
    pub control: AirPumpControlConfig,
}

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum AirPumpControlConfig {
    /// Disabled air pump control.
    #[default]
    Off,
    /// Activate the air pump control pin permanently.
    AlwaysOn { pin: u32 },
}
