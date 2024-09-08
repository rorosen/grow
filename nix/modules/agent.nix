{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.grow-agent;
  airSensorModule = lib.types.submodule (_: {
    options = {
      model = lib.mkOption {
        type = lib.types.enum [ "Bme680" ];
        description = "The model of the air sensor";
      };
      address = lib.mkOption {
        type = lib.types.nonEmptyStr;
        description = "The address of the air sensor";
      };
    };
  });

  lightSensorModule = lib.types.submodule (_: {
    options = {
      model = lib.mkOption {
        type = lib.types.enum [ "Bh1750Fvi" ];
        description = "The model of the light sensor";
      };
      address = lib.mkOption {
        type = lib.types.nonEmptyStr;
        description = "The address of the light sensor";
      };
    };
  });

  waterLevelSensorModule = lib.types.submodule (_: {
    options = {
      model = lib.mkOption {
        type = lib.types.enum [ "Vl53L0X" ];
        description = "The model of the water level sensor";
      };
      address = lib.mkOption {
        type = lib.types.nonEmptyStr;
        description = "The address of the water level sensor";
      };
    };
  });
in
{
  options.services.grow-agent = {
    enable = lib.mkEnableOption "the grow agent.";
    package = lib.mkPackageOption pkgs "grow-agent" { };

    logLevel = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "info";
      example = "debug";
      description = "The rust log level.";
    };

    config = {
      enable = (lib.mkEnableOption "the generation of the agent configuration") // {
        default = true;
      };
      i2c_path = lib.mkOption {
        type = lib.types.nonEmptyStr;
        default = "/dev/i2c-1";
        description = "Path to the I2C device interface.";
      };
      gpio_path = lib.mkOption {
        type = lib.types.nonEmptyStr;
        default = "/dev/gpiochip0";
        description = "Path to the GPIO character device.";
      };
      grow_id = lib.mkOption {
        type = lib.types.nonEmptyStr;
        default = "grow";
        example = "tomatoes";
        description = "The identifier of this grow";
      };

      air = {
        control = {
          mode = lib.mkOption {
            type = lib.types.enum [
              "Off"
              "Cyclic"
            ];
            default = "Off";
            description = "Control mode of the exhaust fan controller.";
          };
          pin = lib.mkOption {
            type = lib.types.ints.u8;
            default = 0;
            example = 25;
            description = "The GPIO pin used to control the exhaust fan.";
          };
          on_duration_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1;
            description = ''
              The duration in seconds for which the exhaust fan control pin should
              be activated (0 means never). Only has an effect in cyclic mode.
            '';
          };
          off_duration_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 0;
            description = ''
              The duration in seconds for which the exhaust fan control pin should
              be deactivated (0 means never). Only has an effect in cyclic mode.
            '';
          };
        };

        sample = {
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the air sensors will be sampled.";
          };
          sensors = lib.mkOption {
            type = lib.types.attrsOf airSensorModule;
            default = { };
            example = lib.literalExpression ''
              {
                left = {
                    model = "Bme680";
                    address = "0x77";
                };
                right = {
                    model = "Bme680";
                    address = "0x76";
                };
              }
            '';
            description = "The air sensors to use.";
          };
        };
      };

      air_pump_control = {
        mode = lib.mkOption {
          type = lib.types.enum [
            "Off"
            "AlwaysOn"
          ];
          default = "Off";
          description = "The air pump control mode.";
        };
        pin = lib.mkOption {
          type = lib.types.ints.u8;
          default = 0;
          example = 24;
          description = "The GPIO pin used to control the air pump.";
        };
      };

      fan = {
        mode = lib.mkOption {
          type = lib.types.enum [
            "Off"
            "Cyclic"
          ];
          default = "Off";
          description = "The fan control mode.";
        };
        pin = lib.mkOption {
          type = lib.types.ints.u8;
          default = 0;
          example = 23;
          description = "The GPIO pin used to control the fan.";
        };
        on_duration_secs = lib.mkOption {
          type = lib.types.ints.unsigned;
          default = 1;
          description = ''
            The duration in seconds for which the fan control pin should be
            activated (0 means never). Only has an effect in cyclic control mode.
          '';
        };
        off_duration_secs = lib.mkOption {
          type = lib.types.ints.unsigned;
          default = 0;
          description = ''
            The duration in seconds for which the fan control pin should be
            deactivated (0 means never). Only has an effect in cyclic control mode.
          '';
        };
      };

      light = {
        control = {
          mode = lib.mkOption {
            type = lib.types.enum [
              "Off"
              "TimeBased"
            ];
            default = "Off";
            description = "The light control mode.";
          };
          pin = lib.mkOption {
            type = lib.types.ints.u8;
            default = 0;
            example = 6;
            description = "The GPIO pin used to control the light.";
          };
          activate_time = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "10:00:00";
            description = "Time of the day to activate the light control pin.";
          };
          deactivate_time = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "22:00:00";
            description = "Time of the day to deactivate the light control pin.";
          };
        };

        sample = {
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the light sensors will be sampled.";
          };
          sensors = lib.mkOption {
            type = lib.types.attrsOf lightSensorModule;
            default = { };
            example = lib.literalExpression ''
              {
                left = {
                    model = "Bh1750Fvi";
                    address = "0x23";
                };
                right = {
                    model = "Bh1750Fvi";
                    address = "0x5C";
                };
              }
            '';
            description = "The light sensors to use.";
          };
        };
      };

      water_level = {
        control = {
          mode = lib.mkOption {
            type = lib.types.enum [ "Off" ];
            default = "Off";
          };
          pumps = lib.mkOption {
            type = with lib.types; attrsOf ints.u8;
            default = { };
            example = lib.literalExpression ''
              {
                main = 17;
              }
            '';
            description = ''
              The label of each water pump with the associated GPIO pin to control the pump.
            '';
          };
        };

        sample = {
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the water level sensors will be sampled.";
          };
          sensors = lib.mkOption {
            type = lib.types.attrsOf waterLevelSensorModule;
            default = { };
            example = lib.literalExpression ''
              {
                main = {
                    model = "Vl53L0X";
                    address = "0x29";
                };
              }
            '';
            description = "The water level sensors to use.";
          };
        };
      };
    };
  };

  config.systemd.services.grow-agent = lib.mkIf cfg.enable {
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "exec";
      ExecStart = "${cfg.package}/bin/grow-agent";
      StateDirectory = "grow";
    };

    environment =
      {
        RUST_LOG = cfg.logLevel;
      }
      // (lib.optionalAttrs cfg.config.enable {
        GROW_AGENT_CONFIG_PATH = pkgs.writeText "grow-agent-config.json" (builtins.toJSON cfg.config);
      });
  };
}
