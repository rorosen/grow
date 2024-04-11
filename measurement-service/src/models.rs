use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable, Selectable};
use grow_utils::api::grow::AirMeasurement;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::air_measurements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StorageAirMeasurement {
    pub measure_time: NaiveDateTime,
    pub left_temperature: Option<f64>,
    pub left_humidity: Option<f64>,
    pub left_pressure: Option<f64>,
    pub left_resistance: Option<f64>,
    pub right_temperature: Option<f64>,
    pub right_humidity: Option<f64>,
    pub right_pressure: Option<f64>,
    pub right_resistance: Option<f64>,
}

fn timestamp_to_storage(value: prost_types::Timestamp) -> Result<NaiveDateTime, String> {
    let secs = value.seconds;
    let nsecs = value.nanos;

    Ok(NaiveDateTime::from_timestamp_opt(secs, nsecs as u32)
        .ok_or(String::from("invalid timestamp"))?)
}

impl TryFrom<AirMeasurement> for StorageAirMeasurement {
    type Error = String;

    fn try_from(value: AirMeasurement) -> Result<Self, Self::Error> {
        let measure_time = match value.measure_time {
            Some(t) => timestamp_to_storage(t)?,
            None => return Err("".into()),
        };

        let (left_temperature, left_humidity, left_pressure, left_resistance) = match value.left {
            Some(s) => (
                Some(s.temperature),
                Some(s.humidity),
                Some(s.pressure),
                Some(s.resistance),
            ),
            None => (None, None, None, None),
        };

        let (right_temperature, right_humidity, right_pressure, right_resistance) =
            match value.right {
                Some(s) => (
                    Some(s.temperature),
                    Some(s.humidity),
                    Some(s.pressure),
                    Some(s.resistance),
                ),
                None => (None, None, None, None),
            };

        Ok(Self {
            measure_time,
            left_temperature,
            left_humidity,
            left_pressure,
            left_resistance,
            right_temperature,
            right_humidity,
            right_pressure,
            right_resistance,
        })
    }
}
