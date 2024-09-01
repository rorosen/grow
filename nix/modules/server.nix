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
      description = "The rust log level.";
    };

    listenAddress = lib.mkOption {
      type = lib.types.nonEmptyStr;
      default = "[::1]:8088";
      example = "192.168.123.123:8088";
      description = "The address on which the server listens.";
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
    };
  };
}
