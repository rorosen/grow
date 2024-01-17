use grow_utils::api::grow::AirSample;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonAirMeasurement {
    #[serde(deserialize_with = "bson::serde_helpers::deserialize_i64_from_bson_datetime")]
    pub time: i64,
    pub left: AirSample,
    pub right: AirSample,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonLightMeasurement {
    #[serde(deserialize_with = "bson::serde_helpers::deserialize_i64_from_bson_datetime")]
    pub time: i64,
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonWaterLevelMeasurement {
    #[serde(deserialize_with = "bson::serde_helpers::deserialize_i64_from_bson_datetime")]
    pub time: i64,
    pub distance: u32,
}
