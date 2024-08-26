CREATE TABLE IF NOT EXISTS light_measurements
(
    id            INTEGER PRIMARY KEY NOT NULL,
    measure_time  INTEGER             NOT NULL,
    label         TEXT                NOT NULL,
    illuminance   REAL                NOT NULL
);
