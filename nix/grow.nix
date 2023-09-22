inputs: let
  inherit (inputs) nixpkgs self;
  pkgs = nixpkgs.legacyPackages.aarch64-linux;

  generateService = name: config:
    (nixpkgs.lib.nixosSystem {
      inherit (pkgs) system;
      modules = [{systemd.services.${name} = config;}];
    })
    .config
    .systemd
    .units
    ."${name}.service"
    .text;

  service = pkgs.writeTextFile {
    name = "grow.service";
    text = generateService "grow" {
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${self.outputs.packages.${pkgs.system}.default}/bin/grow";
      };
      wantedBy = ["multi-user.target"];
    };
  };
in
  pkgs.writeShellScriptBin "activate" ''
    mkdir -p /etc/systemd/system/
    ln -sf ${service} /etc/systemd/system/grow.service
    systemctl daemon-reload
  ''
