use std::{collections::HashMap, net::SocketAddr, str::FromStr};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use bson::{doc, DateTime, Document};
use futures::StreamExt;
use grow_utils::{
    api::grow::AirSample, StorageAirMeasurement, StorageLightMeasurement,
    StorageWaterLevelMeasurement,
};
use mongodb::Database;
use rand::Rng;

use crate::{
    app::{
        AIR_MEASUREMENTS_COLLECTION, LIGHT_MEASUREMENTS_COLLECTION,
        WATER_LEVEL_MEASUREMENTS_COLLECTION,
    },
    datasource::types::JsonAirMeasurement,
    error::AppError,
};

use self::types::{JsonLightMeasurement, JsonWaterLevelMeasurement};

mod types;

// parse-json
// | project "time", "ltemp"="left.temperature", "left humidity"="left.humidity", "left pressure"="left.pressure", "left air resistance"="left.resistance", "right temperature"="right.temperature", "right humidity"="right.humidity", "right pressure"="right.pressure", "right air resistance"="right.resistance"

pub struct Datasource {
    db: Database,
}

impl Datasource {
    pub async fn new(db: Database) -> Self {
        let t = DateTime::now().timestamp_millis();
        let c = (0..3000i64).rev().map(|n| StorageAirMeasurement {
            measure_time: DateTime::from_millis(t - n * 300_000),
            left: Some(AirSample {
                temperature: rand::thread_rng().gen_range(16.0..21.0),
                humidity: rand::thread_rng().gen_range(45.0..55.0),
                pressure: rand::thread_rng().gen_range(950.0..1020.0),
                resistance: rand::thread_rng().gen_range(20000.0..31000.0),
            }),
            right: Some(AirSample {
                temperature: rand::thread_rng().gen_range(16.0..21.0),
                humidity: rand::thread_rng().gen_range(45.0..55.0),
                pressure: rand::thread_rng().gen_range(950.0..1020.0),
                resistance: rand::thread_rng().gen_range(20000.0..31000.0),
            }),
        });

        db.collection::<StorageAirMeasurement>(AIR_MEASUREMENTS_COLLECTION)
            .insert_many(c, None)
            .await
            .unwrap();

        let t = DateTime::now().timestamp_millis();
        let c = (0..3000i64).rev().map(|n| StorageLightMeasurement {
            measure_time: DateTime::from_millis(t - n * 300_000),
            left: rand::thread_rng().gen_range(10000.0..20000.0),
            right: rand::thread_rng().gen_range(10000.0..20000.0),
        });

        db.collection::<StorageLightMeasurement>(LIGHT_MEASUREMENTS_COLLECTION)
            .insert_many(c, None)
            .await
            .unwrap();

        let t = DateTime::now().timestamp_millis();
        let c = (0..3000i64).rev().map(|n| StorageWaterLevelMeasurement {
            measure_time: DateTime::from_millis(t - n * 300_000),
            distance: rand::thread_rng().gen_range(100..150),
        });

        db.collection::<StorageWaterLevelMeasurement>(WATER_LEVEL_MEASUREMENTS_COLLECTION)
            .insert_many(c, None)
            .await
            .unwrap();

        Self { db }
    }

    pub async fn run(self, address: String, port: u16) -> Result<(), AppError> {
        let addr =
            SocketAddr::from_str(&format!("{address}:{port}")).map_err(AppError::AddrParse)?;

        let router = Router::new()
            .route("/air", get(air))
            .route("/light", get(light))
            .route("/water-level", get(water_level))
            .with_state(self.db);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        axum::serve(listener, router)
            .await
            .map_err(AppError::DatasourceError)
    }
}

async fn air(
    State(db): State<Database>,
    Query(params): Query<HashMap<String, i64>>,
) -> Result<Json<Vec<JsonAirMeasurement>>, StatusCode> {
    let Some((from, to, interval)) = decode_params(&params) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let pipeline = make_pipeline(from, to, interval);
    let mut cursor = db
        .collection::<StorageAirMeasurement>(AIR_MEASUREMENTS_COLLECTION)
        .aggregate(pipeline, None)
        .await
        .map_err(|e| {
            log::error!("failed to run air measurements aggregation operation: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // TODO: make this work with try_collect
    let mut measurements = Vec::new();
    while let Some(measurement) = cursor.next().await {
        let measurement = measurement.map_err(|e| {
            log::error!("failed to get air measurement: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let measurement: JsonAirMeasurement = bson::from_document(measurement).map_err(|e| {
            log::error!("failed to deserialize air measurement: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        measurements.push(measurement);
    }

    Ok(Json(measurements))
}

async fn light(
    State(db): State<Database>,
    Query(params): Query<HashMap<String, i64>>,
) -> Result<Json<Vec<JsonLightMeasurement>>, StatusCode> {
    let Some((from, to, interval)) = decode_params(&params) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let pipeline = make_pipeline(from, to, interval);
    let mut cursor = db
        .collection::<StorageLightMeasurement>(LIGHT_MEASUREMENTS_COLLECTION)
        .aggregate(pipeline, None)
        .await
        .map_err(|e| {
            log::error!("failed to run light measurements aggregation operation: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // TODO: make this work with try_collect
    let mut measurements = Vec::new();
    while let Some(measurement) = cursor.next().await {
        let measurement = measurement.map_err(|e| {
            log::error!("failed to get light measurement: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let measurement: JsonLightMeasurement = bson::from_document(measurement).map_err(|e| {
            log::error!("failed to deserialize light measurement: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        measurements.push(measurement);
    }

    Ok(Json(measurements))
}

async fn water_level(
    State(db): State<Database>,
    Query(params): Query<HashMap<String, i64>>,
) -> Result<Json<Vec<JsonWaterLevelMeasurement>>, StatusCode> {
    let Some((from, to, interval)) = decode_params(&params) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let pipeline = make_pipeline(from, to, interval);
    let mut cursor = db
        .collection::<StorageWaterLevelMeasurement>(WATER_LEVEL_MEASUREMENTS_COLLECTION)
        .aggregate(pipeline, None)
        .await
        .map_err(|e| {
            log::error!("failed to run water level measurements aggregation operation: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // TODO: make this work with try_collect
    let mut measurements = Vec::new();
    while let Some(measurement) = cursor.next().await {
        let measurement = measurement.map_err(|e| {
            log::error!("failed to get water level measurement: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let measurement: JsonWaterLevelMeasurement =
            bson::from_document(measurement).map_err(|e| {
                log::error!("failed to deserialize water level measurement: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        measurements.push(measurement);
    }

    Ok(Json(measurements))
}

fn decode_params(params: &HashMap<String, i64>) -> Option<(i64, i64, i64)> {
    let Some(from) = params.get("from") else {
        return None;
    };

    let Some(to) = params.get("to") else {
        return None;
    };

    let Some(interval) = params.get("interval") else {
        return None;
    };

    Some((*from, *to, *interval))
}

fn make_pipeline(from: i64, to: i64, interval: i64) -> Vec<Document> {
    let match_stage = doc! {
        "$match": {
            "measure_time": {"$gte": DateTime::from_millis(from), "$lte": DateTime::from_millis(to)}
        }
    };
    let group_stage = doc! {
        "$group":{
            "_id": {"$subtract": [
                {"$toLong": "$measure_time"},
                {"$mod": [{"$toLong": "$measure_time"}, interval]}
            ]},
            "time": {"$last": "$measure_time"},
            "left": {"$last": "$left"},
            "right": {"$last": "$right"}
        }
    };
    let sort_stage = doc! {
        "$sort": {"time": 1}
    };

    vec![match_stage, group_stage, sort_stage]
}
