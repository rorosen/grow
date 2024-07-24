use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize)]
pub struct AirPumpControlConfig {
    /// Whether to enable the air pump controller.
    pub enable: bool,
    /// The gpio pin used to control the air pump.
    pub pin: u8,
}
