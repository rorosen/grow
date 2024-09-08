{
  inputs,
  pkgs,
  config,
  modulesPath,
  ...
}:
{
  imports = [
    "${modulesPath}/installer/sd-card/sd-image-aarch64.nix"
    inputs.nixos-hardware.nixosModules.raspberry-pi-4
    # Import all grow modules
  ] ++ (builtins.attrValues inputs.grow.nixosModules);

  system.stateVersion = "24.05";
  sdImage.compressImage = false;

  # Place your SSH key(s) here
  users.users.root.openssh.authorizedKeys.keys = [ "" ];

  # Overlay the grow packages
  nixpkgs.overlays =
    let
      packages = inputs.grow.packages.x86_64-linux;
    in
    [
      (final: super: {
        makeModulesClosure = x: super.makeModulesClosure (x // { allowMissing = true; });
      })
      (_: prev: {
        grow-agent = packages.agent;
        grow-server = packages.server;
        grow-sensortest = packages.sensortest;
        grafanaPlugins = prev.grafanaPlugins // {
          inherit (packages) yesoreyeram-infinity-datasource;
        };
      })
    ];

  # Install the grow-sensortest program
  environment.systemPackages = [ pkgs.grow-sensortest ];

  # Enable I2C
  hardware.raspberry-pi."4" = {
    i2c1.enable = true;
  };

  networking = {
    hostName = "growPi";
    firewall.allowedTCPPorts = [ config.services.grafana.settings.server.http_port ];
  };

  services = {
    openssh.enable = true;

    grow-grafana.enable = true;
    grow-server.enable = true;
    grow-agent = {
      enable = true;
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
