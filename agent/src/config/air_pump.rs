use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub enum AirPumpControlMode {
    /// Disabled air pump control.
    #[default]
    Off,
    /// Activate the air pump permanently.
    Permanent,
}

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct AirPumpControlConfig {
    /// The control mode.
    pub mode: AirPumpControlMode,
    /// The gpio pin used to control the air pump.
    #[serde(default)]
    pub pin: u32,
}
