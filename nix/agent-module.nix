{
  pkgs,
  config,
  lib,
  ...
}:
let
  cfg = config.grow.agent;
in
{
  options.grow.agent = {
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
            type = lib.types.int.u8;
            default = 6;
            description = "Gpio pin to control the light.";
          };
          activateTime = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "10:00:00";
            description = "Time of the day to switch on the light.";
          };
          deactivateTime = lib.mkOption {
            type = lib.tpes.nonEmptyStr;
            default = "04:00:00";
            description = "Time of the day to switch off the light.";
          };
        };
        sample = {
          leftAddress = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x23";
            description = "Address of the left light sensor.";
          };
          rightAddress = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x5C";
            description = "Address of the right light sensor.";
          };
          sampleRateSecs = lib.mkOption {
            type = lib.types.int.unsigned;
            default = 1800;
            description = "Rate in which the light sensors will be sampled.";
          };
        };
      };

      waterLevel = {
        control = {
          enable = lib.mkEnableOption "the pump controller.";
          pin = lib.mkOption {
            type = lib.types.int.u8;
            default = 17;
            description = "Gpio pin to control the water pump.";
          };
        };
        sample = {
          sensorAddress = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x29";
            description = "Address of the water level sensor.";
          };
          sampleRateSecs = lib.mkOption {
            type = lib.types.int.unsigned;
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
          type = lib.types.int.u8;
          default = 23;
          description = "Gpio pin to control the fans.";
        };
        onDurationSecs = lib.mkOption {
          type = lib.types.int.unsigned;
          default = 1;
          description = ''
            The duration in seconds for which the circulation fans should
            run (0 means never). Only has an effect if control is enabled.'';
        };
        offDurationSecs = lib.mkOption {
          type = lib.types.int.unsigned;
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
            type = lib.types.int.u8;
            default = 25;
            description = "Gpio pin to control the exhaust fan.";
          };
          onDurationSecs = lib.mkOption {
            type = lib.types.int.unsigned;
            default = 1;
            description = ''
              The duration in seconds for which the exhaust fan should
              run (0 means never). Only has an effect in cyclic mode.
            '';
          };
          offDurationSecs = lib.mkOption {
            type = lib.types.int.unsigned;
            default = 0;
            description = ''
              The duration in seconds for which the exhaust fan should be
              stopped (0 means never). Only has an effect in cyclic mode.
            '';
          };
        };
        sample = {
          leftAddress = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x77";
            description = "Address of the left air sensor.";
          };
          rightAddress = lib.mkOption {
            type = lib.types.nonEmptyStr;
            default = "0x76";
            description = "Address of the right air sensor.";
          };
          sampleRateSecs = lib.mkOption {
            type = lib.types.int.unsigned;
            default = 1800;
            description = "Rate in which the air sensor will be sampled.";
          };
        };
      };

      airPump = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          description = "Whether to enable the air pump controller.";
        };
        pin = lib.mkOption {
          type = lib.types.int.u8;
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
