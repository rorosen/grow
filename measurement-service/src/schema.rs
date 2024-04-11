diesel::table! {
    air_measurements (measure_time) {
        measure_time -> Timestamp,
        left_temperature -> Nullable<Double>,
        left_humidity -> Nullable<Double>,
        left_pressure -> Nullable<Double>,
        left_resistance -> Nullable<Double>,
        right_temperature -> Nullable<Double>,
        right_humidity -> Nullable<Double>,
        right_pressure -> Nullable<Double>,
        right_resistance -> Nullable<Double>,
    }
}
