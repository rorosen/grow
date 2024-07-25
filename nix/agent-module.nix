{
  pkgs,
  config,
  lib,
  ...
}:
let
  cfg = config.services.grow-agent;
in
{
  options.services.grow-agent = {
    enable = lib.mkEnableOption "the grow agent.";
    logLevel = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "info";
      description = "The rust log level.";
    };
    config = {
      light = {
        control = {
          enable = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = "Whether to enable the light controller.";
          };
          pin = lib.mkOption {
            type = lib.types.ints.u8;
            default = 6;
            description = "Gpio pin to control the light.";
          };
          activate_time = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "10:00:00";
            description = "Time of the day to switch on the light.";
          };
          deactivate_time = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "04:00:00";
            description = "Time of the day to switch off the light.";
          };
        };
        sample = {
          left_address = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x23";
            description = "Address of the left light sensor.";
          };
          right_address = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x5C";
            description = "Address of the right light sensor.";
          };
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the light sensors will be sampled.";
          };
        };
      };

      water_level = {
        control = {
          enable = lib.mkEnableOption "the pump controller.";
          pin = lib.mkOption {
            type = lib.types.ints.u8;
            default = 17;
            description = "Gpio pin to control the water pump.";
          };
        };
        sample = {
          sensor_address = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x29";
            description = "Address of the water level sensor.";
          };
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the water level sensor will be sampled.";
          };
        };
      };

      fan = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = "Whether to enable the fan controller.";
        };
        pin = lib.mkOption {
          type = lib.types.ints.u8;
          default = 23;
          description = "Gpio pin to control the fans.";
        };
        on_duration_secs = lib.mkOption {
          type = lib.types.ints.unsigned;
          default = 1;
          description = ''
            The duration in seconds for which the circulation fans should
            run (0 means never). Only has an effect if control is enabled.'';
        };
        off_duration_secs = lib.mkOption {
          type = lib.types.ints.unsigned;
          default = 0;
          description = ''
            The duration in seconds for which the circulation fans should be
            stopped (0 means never). Only has an effect if control is enabled.
          '';
        };
      };

      air = {
        control = {
          mode = lib.mkOption {
            type = lib.types.enum [
              "Off"
              "Cyclic"
              "Threshold"
            ];
            default = "Cyclic";
            description = "Control mode of the exhaust fan controller.";
          };
          pin = lib.mkOption {
            type = lib.types.ints.u8;
            default = 25;
            description = "Gpio pin to control the exhaust fan.";
          };
          on_duration_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1;
            description = ''
              The duration in seconds for which the exhaust fan should
              run (0 means never). Only has an effect in cyclic mode.
            '';
          };
          off_duration_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 0;
            description = ''
              The duration in seconds for which the exhaust fan should be
              stopped (0 means never). Only has an effect in cyclic mode.
            '';
          };
        };
        sample = {
          left_address = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x77";
            description = "Address of the left air sensor.";
          };
          right_address = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x76";
            description = "Address of the right air sensor.";
          };
          sample_rate_secs = lib.mkOption {
            type = lib.types.ints.unsigned;
            default = 1800;
            description = "Rate in which the air sensor will be sampled.";
          };
        };
      };

      air_pump = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = "Whether to enable the air pump controller.";
        };
        pin = lib.mkOption {
          type = lib.types.ints.u8;
          default = 24;
          description = "Gpio pin to control the air pump.";
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.grow-agent = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "exec";
        ExecStart = "${pkgs.grow-agent}/bin/grow-agent";
      };
      environment = {
        RUST_LOG = cfg.logLevel;
        GROW_AGENT_CONFIG_PATH = pkgs.writeText "grow-agent-config.json" (builtins.toJSON cfg.config);
      };
    };
  };
}
