CREATE TABLE IF NOT EXISTS water_level_measurements
(
    id            INTEGER PRIMARY KEY NOT NULL,
    measure_time  INTEGER             NOT NULL,
    label         TEXT                NOT NULL,
    distance      INTEGER             NOT NULL
);
