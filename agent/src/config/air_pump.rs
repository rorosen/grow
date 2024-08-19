use serde::Deserialize;

#[derive(PartialEq, Debug, Default, Deserialize)]
pub struct AirPumpControlConfig {
    /// Whether to enable the air pump controller.
    #[serde(default)]
    pub enable: bool,
    /// The gpio pin used to control the air pump.
    #[serde(default)]
    pub pin: u8,
}
