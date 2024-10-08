{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.grow-agent;
  mkControlOptions =
    {
      feedbackControl ? false,
    }:
    {
      mode = lib.mkOption {
        type =
          lib.types.enum [
            "Off"
            "Cyclic"
            "TimeBased"
          ]
          ++ (lib.optional feedbackControl "Feedback");
        example = "Cyclic";
        default = "Off";
      };
      pin = lib.mkOption {
        type = lib.types.ints.u8;
        example = 24;
        description = "The GPIO pin used for control.";
      };
      on_duration_secs = lib.mkOption {
        type = lib.types.ints.unsigned;
        example = 300;
        description = ''
          The duration in seconds for which the control pin should be activated
          (0 means never). Only has an effect in cyclic mode.
        '';
      };
      off_duration_secs = lib.mkOption {
        type = lib.types.ints.unsigned;
        example = 600;
        description = ''
          The duration in seconds for which the control pin should be Deactivated
          (0 means never). Only has an effect in cyclic mode.
        '';
      };
      activate_time = lib.mkOption {
        type = lib.types.nonEmptyStr;
        example = "10:00:00";
        description = "Time of the day to activate the control pin.";
      };
      deactivate_time = lib.mkOption {
        type = lib.types.nonEmptyStr;
        example = "22:00:00";
        description = "Time of the day to deactivate the control pin.";
      };
      activate_condition = lib.mkOption {
        type = lib.types.nonEmptyStr;
        example = "some_value > 100";
        description = "The condition that activates the control pin.";
      };
      deactivate_condition = lib.mkOption {
        type = lib.types.nonEmptyStr;
        example = "another_value <= 69";
        description = "The condition that deactivates the control pin.";
      };
    };

  mkSampleOptions = models: {
    sample_rate_secs = lib.mkOption {
      type = lib.types.ints.unsigned;
      example = 1800;
      description = "Rate in which the sensors will be sampled.";
    };
    sensors = lib.mkOption {
      type =
        with lib.types;
        attrsOf (submodule {
          options = {
            model = lib.mkOption {
              type = lib.types.enum models;
              description = "The model of the sensor";
            };
            address = lib.mkOption {
              type = lib.types.nonEmptyStr;
              description = "The address of the sensor";
            };
          };
        });
      default = { };
      example = lib.literalExpression ''
        {
          left = {
              model = "some_model";
              address = "0x79";
          };
          right = {
              model = "another_model";
              address = "0x46";
          };
        }
      '';
      description = "The sensors to use.";
    };
  };

in
{
  options.services.grow-agent = {
    enable = lib.mkEnableOption "the grow agent";
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
        control = mkControlOptions { feedbackControl = true; };
        sample = mkSampleOptions [ "Bme680" ];
      };

      air_pump.control = mkControlOptions;
      fan.control = mkControlOptions;

      light = {
        sample = mkSampleOptions [ "Bh1750Fvi" ];
        control = mkControlOptions;
      };

      water_level = {
        sample = mkSampleOptions [ "Vl53L0X" ];
        control = mkControlOptions { feedbackControl = true; };
      };
    };
  };

  config.systemd.services.grow-agent =
    let
      mkControlConfig =
        opts:
        if opts.mode == "TimeBased" then
          {
            inherit (opts)
              mode
              pin
              activate_time
              deactivate_time
              ;
          }
        else if opts.mode == "Cyclic" then
          {
            inherit (opts)
              mode
              pin
              on_duration_secs
              off_duration_secs
              ;
          }
        else if opts.mode == "Feedback" then
          {
            inherit (opts)
              mode
              pin
              activate_condition
              deactivate_condition
              ;
          }
        else
          { inherit (opts) mode; };

      mkSampleConfig =
        opts: lib.optionalAttrs (opts.sensors != { }) { inherit (opts) sample_rate_secs sensors; };

      agentConfig = {
        air = {
          control = mkControlConfig cfg.config.air.control;
          sample = mkSampleConfig cfg.config.air.sample;
        };
        air_pump.control = mkControlConfig cfg.config.air_pump.control;
        fan.control = mkControlConfig cfg.config.fan.control;
        light = {
          control = mkControlConfig cfg.config.light.control;
          sample = mkSampleConfig cfg.config.light.sample;
        };
        water_level = {
          control = mkControlConfig cfg.config.water_level.control;
          sample = mkSampleConfig cfg.config.water_level.sample;
        };
      };
    in
    lib.mkIf cfg.enable {
      wantedBy = [ "multi-user.target" ];
      after = [ "time-sync.target" ];
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
          GROW_AGENT_CONFIG_PATH = pkgs.writeText "grow-agent-config.json" (builtins.toJSON agentConfig);
        });
    };
}
