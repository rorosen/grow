# Grow Agent

The agent interfaces with hardware and stores measurements in local databases. See the
[agent module](../nix/modules/agent.nix) for available options.

## Sensor types

- **Air**: Measures attributes of the air, e.g. temperature, humidity, pressure.
- **Light**: Measures attributes of the light, e.g. illuminance.
- **Water Level**: Measures the water fill level in hydroponic
  [deep water culture](https://en.wikipedia.org/wiki/Deep_water_culture) setups, e.g. the distance
  to the water surface.

Currently only a few sensors are supported.

| Type        | Model                                                                                                    | Comment                                                  |
| ----------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| Air         | [BME680](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bme680-ds001.pdf) | Low power gas, pressure, temperature & humidity sensor   |
| Light       | [BH1750FVI](https://www.mouser.com/datasheet/2/348/bh1750fvi-e-186247.pdf)                               | Digital 16bit Serial Output Type Ambient Light Sensor IC |
| Water Level | [Vl53L0X](https://www.st.com/resource/en/datasheet/vl53l0x.pdf)                                          | Time-of-Flight ranging sensor                            |

## Configuration

The agent requires a configuration file and errors if it can't find any. Specify the config by
setting the `GROW_AGENT_CONFIG_PATH` environment variable. If no config path is specified, the agent
falls back to read the config from `${CONFIGURATION_DIRECTORY}/config.json`. You can always print
the default configuration via the following command.

```shell
nix run github:rorosen/grow#agent -- --print-default-config
```

However, notice that the default configuration does exactly nothing as everything is disabled by
default. It can be used as a template for a real configuration. You can omit any item of the config
that you don't want to configure, meaning that the following configurations are equivalent.

<details>
<summary>Verbose configuration</summary>

```json
{
  "i2c_path": "/dev/i2c-1",
  "gpio_path": "/dev/gpiochip0",
  "grow_id": "grow",
  "air": {
    "control": {
      "mode": "Cyclic",
      "pin": 25,
      "on_duration_secs": 600,
      "off_duration_secs": 1800
    },
    "sample": {
      "sample_rate_secs": 1800,
      "sensors": {
        "left": {
          "address": "0x77",
          "model": "Bme680"
        },
        "right": {
          "address": "0x76",
          "model": "Bme680"
        }
      }
    }
  },
  "air_pump_control": {
    "mode": "Off",
    "pin": 0
  },
  "fan": {
    "mode": "Off",
    "pin": 0,
    "on_duration_secs": 0,
    "off_duration_secs": 0
  },
  "light": {
    "control": {
      "mode": "TimeBased",
      "pin": 6,
      "activate_time": "10:00:00",
      "deactivate_time": "22:00:00"
    },
    "sample": {
      "sample_rate_secs": 0,
      "sensors": {}
    }
  },
  "water_level": {
    "control": {
      "mode": "Off",
      "pumps": {}
    },
    "sample": {
      "sample_rate_secs": 0,
      "sensors": {}
    }
  }
}
```

</details>

<details>
<summary>Minimal configuration</summary>

```json
{
  "air": {
    "control": {
      "mode": "Cyclic",
      "pin": 25,
      "on_duration_secs": 600,
      "off_duration_secs": 1800
    },
    "sample": {
      "sample_rate_secs": 1800,
      "sensors": {
        "left": {
          "address": "0x77",
          "model": "Bme680"
        },
        "right": {
          "address": "0x76",
          "model": "Bme680"
        }
      }
    }
  },
  "light": {
    "control": {
      "mode": "TimeBased",
      "pin": 6,
      "activate_time": "10:00:00",
      "deactivate_time": "22:00:00"
    }
  }
}
```

</details>

This does the following:

- The air controller activates GPIO pin 25, which should control an exhaust ventilator, every 30
  minutes for 10 minutes
- The light controller activates GPIO pin 6, which should control a plant lamp, at 10:00:00 UTC and
  deactivates it at 22:00:00 UTC
- The air sampler measures every 30 minutes with two BME680 sensors
