{
  config,
  lib,
  pkgs,
}:
let
  cfg = config.grow.grafana;
in
{
  options.grow.grafana = {
    enable = lib.mkEnableOption "the grow grafana instance.";
  };

  config.services.grafama = lib.mkIf cfg.enable {
    services.grafana = {
      enable = true;
      settings.server = {
        protocol = "http";
        http_addr = "::";
        http_port = 3000;
      };
    };
  };
}
