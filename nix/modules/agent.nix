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
        type = lib.types.enum (
          [
            "Off"
            "Cyclic"
            "TimeBased"
          ]
          ++ (lib.optional feedbackControl "Feedback")
        );
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

  sampleOptions = {
    mode = lib.mkOption {
      type = lib.types.enum [
        "Off"
        "Interval"
      ];
      example = "Interval";
      default = "Off";
    };
    period = lib.mkOption {
      type = lib.types.nonEmptyStr;
      example = "10m30s";
      description = "The period between two measurements";
    };
    script_path = lib.mkOption {
      type = with lib.types; either nonEmptyStr path;
      example = "/path/to/script.py";
      description = "Path to the python script that takes the measurement";
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
        sample = sampleOptions;
      };

      air_pump.control = mkControlOptions { };
      fan.control = mkControlOptions { };

      light = {
        sample = sampleOptions;
        control = mkControlOptions { };
      };

      water_level = {
        sample = sampleOptions;
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
        opts:
        if opts.mode == "Interval" then
          {
            inherit (opts) mode period script_path;
          }
        else
          { inherit (opts) mode; };

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
