# Grow Sensortest

A simple Command line tool that tests grow sensors.

```shell
# Usage: grow-sensortest <variant> <sensor_address>
# Measure with a sensor of model Vl53L0X at address 0x23
$ grow-sensortest vl53l0x 0x23
LightMeasurement { measure_time: 1725749724, label: "test", illuminance: 69.0 }
```
