{
  pkgs,
  nixosSystem,
  agent,
}: let
  generateService = name: config:
    (nixosSystem {
      inherit (pkgs) system;
      modules = [
        {
          system.stateVersion = "23.11";
          systemd.services.${name} = config;
        }
      ];
    })
    .config
    .systemd
    .units
    ."${name}.service"
    .text;

  service = pkgs.writeTextFile {
    name = "grow-agent.service";
    text = generateService "grow-agent" {
      serviceConfig = {
        Type = "exec";
        ExecStart = "${agent}/bin/grow-agent";
      };
      environment = {
        RUST_LOG = "info";

        # control
        GROW_AGENT_AIR_PUMP_CONTROL_DISABLED = "false";
        GROW_AGENT_AIR_PUMP_CONTROL_PIN = "24";

        GROW_AGENT_EXHAUST_CONTROL_MODE = "cyclic";
        GROW_AGENT_EXHAUST_CONTROL_PIN = "25";
        # uncomment for fast mode
        # GROW_AGENT_EXHAUST_CONTROL_PIN= "26";
        GROW_AGENT_EXHAUST_CONTROL_ON_DURATION_SECS = "1";
        GROW_AGENT_EXHAUST_CONTROL_OFF_DURATION_SECS = "0";

        GROW_AGENT_FAN_CONTROL_MODE = "cyclic";
        GROW_AGENT_FAN_CONTROL_PIN = "23";
        GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS = "1";
        GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS = "0";

        GROW_AGENT_LIGHT_CONTROL_MODE = "time";
        GROW_AGENT_LIGHT_CONTROL_PIN = "6";
        GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_TIME = "10:00";
        GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_TIME = "22:00";

        GROW_AGENT_PUMP_CONTROL_DISABLE = "true";
        GROW_AGENT_PUMP_CONTROL_LEFT_PIN = "17";
        GROW_AGENT_PUMP_CONTROL_RIGHT_PIN = "22";

        # sample
        GROW_AGENT_AIR_SAMPLE_LEFT_SENSOR_ADDRESS = "0x76";
        GROW_AGENT_AIR_SAMPLE_RIGHT_SENSOR_ADDRESS = "0x77";
        GROW_AGENT_AIR_SAMPLE_RATE_SECS = "300";

        GROW_AGENT_LIGHT_SAMPLE_LEFT_SENSOR_ADDRESS = "0x5C";
        GROW_AGENT_LIGHT_SAMPLE_RIGHT_SENSOR_ADDRESS = "0x23";
        GROW_AGENT_LIGHT_SAMPLE_RATE_SECS = "300";

        GROW_AGENT_WATER_LEVEL_SAMPLE_LEFT_SENSOR_ADDRESS = "0x29";
        GROW_AGENT_WATER_LEVEL_SAMPLE_RIGHT_SENSOR_ADDRESS = "0x2A";
        GROW_AGENT_WATER_LEVEL_SAMPLE_RATE_SECS = "300";
      };
      wantedBy = ["multi-user.target"];
    };
  };
in
  pkgs.writeShellScriptBin "activate" ''
    mkdir -p /etc/systemd/system/
    ln -sf ${service} /etc/systemd/system/grow-agent.service
    ln -sf ${service} /etc/systemd/system/multi-user.target.wants/grow-agent.service
    systemctl daemon-reload
  ''
