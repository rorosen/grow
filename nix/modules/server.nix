{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.grow-server;
in
{
  options.services.grow-server = {
    enable = lib.mkEnableOption "the grow server.";
    package = lib.mkPackageOption pkgs "grow-server" { };

    logLevel = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "info";
      example = "debug";
      description = "The rust log level.";
    };

    listenAddress = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "::1";
      example = "192.168.123.123";
      description = "The address to listen on.";
    };

    listenPort = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "The port to listen on.";

    };
  };

  config.systemd.services.grow-server = lib.mkIf cfg.enable {
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "exec";
      ExecStart = "${cfg.package}/bin/grow-server";
      StateDirectory = "grow";
    };

    environment = {
      RUST_LOG = cfg.logLevel;
      GROW_LISTEN_ADDRESS = cfg.listenAddress;
      GROW_LISTEN_PORT = builtins.toString cfg.listenPort;
    };
  };
}
