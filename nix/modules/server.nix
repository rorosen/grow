{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.grow.server;
in
{
  options.grow.server = {
    enable = lib.mkEnableOption "the grow server.";

    logLevel = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "info";
      example = "debug";
      description = "The rust log level.";
    };

    listenAddress = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "[::1]";
      example = "192.168.123.123";
      description = "The address on which the server listens.";
    };

    listenPort = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "The port on which the server listens.";

    };
  };

  config.systemd.services.grow-server = {
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "exec";
      ExecStart = "${pkgs.grow-server}/bin/grow-server";
      StateDirectory = "grow";
    };

    environment = {
      RUST_LOG = cfg.logLevel;
      GROW_LISTEN_ADDRESS = cfg.listenAddress;
      GROW_LISTEN_PORT = builtins.toString cfg.listenPort;
    };
  };
}
