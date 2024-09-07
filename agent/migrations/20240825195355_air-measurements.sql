CREATE TABLE IF NOT EXISTS air_measurements
(
    id            INTEGER PRIMARY KEY NOT NULL,
    measure_time  INTEGER             NOT NULL,
    label         TEXT                NOT NULL,
    temperature   REAL,
    humidity      REAL,
    pressure      REAL,
    resistance    REAL
);
