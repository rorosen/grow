{
  inputs,
  config,
  modulesPath,
  ...
}:
let
  growPkgs = inputs.grow.packages.x86_64-linux;
in
{
  imports = [
    "${modulesPath}/installer/sd-card/sd-image-aarch64.nix"
    inputs.nixos-hardware.nixosModules.raspberry-pi-4
    # Import all grow modules
  ] ++ (builtins.attrValues inputs.grow.nixosModules);

  system.stateVersion = "24.05";
  sdImage.compressImage = false;

  # Place your SSH key(s) here
  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE8BYhhtM7cj2GqBtW3ftPGtlBazkpePGrMSQX4MG2QD rob@hp"
  ];

  # https://github.com/NixOS/nixpkgs/issues/154163#issuecomment-1350599022
  nixpkgs.overlays = [
    (_final: prev: { makeModulesClosure = x: prev.makeModulesClosure (x // { allowMissing = true; }); })
  ];

  # Install the grow-sensortest program
  environment.systemPackages = [ growPkgs.sensortest-aarch64 ];

  # Enable I2C
  hardware.raspberry-pi."4" = {
    i2c1.enable = true;
  };

  networking = {
    hostName = "growPi";
    # Open the Grafana HTTP port
    firewall.allowedTCPPorts = [ config.services.grafana.settings.server.http_port ];
  };

  services = {
    openssh.enable = true;

    grow-grafana = {
      enable = true;
      provision.datasource.package = growPkgs.yesoreyeram-infinity-datasource-aarch64;
    };
    grow-server = {
      enable = true;
      package = growPkgs.server-aarch64;
    };
    grow-agent = {
      enable = true;
      package = growPkgs.agent-aarch64;

      config = {
        air.sample = {
          sample_rate_secs = 600;
          sensors.main = {
            model = "Bme680";
            address = "0x77";
          };
        };

        light = {
          control = {
            mode = "TimeBased";
            pin = 24;
            activate_time = "10:00:00";
            deactivate_time = "04:00:00";
          };
          sample = {
            sample_rate_secs = 1200;
            sensors.main = {
              model = "Bh1750Fvi";
              address = "0x23";
            };
          };
        };
      };
    };
  };
}
