#[derive(Debug, Default)]
pub struct LightMeasurement {
    pub lux: f64,
}

#[derive(Debug, Default)]
pub struct WaterLevelMeasurement {
    pub distance: u16,
}

#[derive(Debug, Default)]
pub struct AirMeasurement {
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub resistance: f64,
}
