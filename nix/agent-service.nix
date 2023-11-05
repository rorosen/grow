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
        Type = "simple";
        ExecStart = "${agent}/bin/grow-agent";
      };
      environment = {
        RUST_LOG = "debug";

        GROW_AGENT_EXHAUST_CONTROL_DISABLE = "false";
        GROW_AGENT_EXHAUST_CONTROL_PIN_SLOW = "25";
        GROW_AGENT_EXHAUST_CONTROL_PIN_FAST = "26";
        GROW_AGENT_EXHAUST_CONTROL_ON_DURATION_SECS = "1";
        GROW_AGENT_EXHAUST_CONTROL_OFF_DURATION_SECS = "0";

        GROW_AGENT_FAN_CONTROL_DISABLE = "false";
        GROW_AGENT_FAN_CONTROL_PIN = "23";
        GROW_AGENT_FAN_CONTROL_ON_DURATION_SECS = "1";
        GROW_AGENT_FAN_CONTROL_OFF_DURATION_SECS = "0";

        GROW_AGENT_AIR_PUMP_DISABLE = "false";
        GROW_AGENT_AIR_PUMP_PIN = "24";

        GROW_AGENT_LIGHT_CONTROL_DISABLE = "false";
        GROW_AGENT_LIGHT_CONTROL_PIN = "6";
        GROW_AGENT_LIGHT_CONTROL_SWITCH_ON_HOUR = "10:00";
        GROW_AGENT_LIGHT_CONTROL_SWITCH_OFF_HOUR = "22:00";

        GROW_AGENT_PUMP_CONTROL_DISABLE = "true";
        GROW_AGENT_PUMP_CONTROL_PIN_LEFT = "17";
        GROW_AGENT_PUMP_CONTROL_PIN_RIGHT = "22";
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
