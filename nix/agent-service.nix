inputs: let
  inherit (inputs) nixpkgs self;
  pkgs = nixpkgs.legacyPackages.aarch64-linux;

  generateService = name: config:
    (nixpkgs.lib.nixosSystem {
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
        Type = "oneshot";
        ExecStart = "${self.outputs.packages.${pkgs.system}.agent}/bin/grow-agent";
      };
      wantedBy = ["multi-user.target"];
    };
  };
in
  pkgs.writeShellScriptBin "activate" ''
    mkdir -p /etc/systemd/system/
    ln -sf ${service} /etc/systemd/system/grow-agent.service
    systemctl daemon-reload
  ''
