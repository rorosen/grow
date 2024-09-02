{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.grow.grafana;
in
{
  options.grow.grafana = {
    enable = lib.mkEnableOption "the grow grafana instance.";
  };

  config.services.grafana = lib.mkIf cfg.enable {
    enable = true;
    declarativePlugins = [ pkgs.grafanaPlugins.yesoreyeram-infinity-datasource ];
    settings.server = {
      protocol = "http";
      http_addr = "::";
      http_port = 3000;
    };

    provision = {
      enable = true;
      datasources.settings.datasources = [
        {
          name = "yesoreyeram-infinity-datasource";
          type = "yesoreyeram-infinity-datasource";
        }
      ];
      dashboards.settings.providers = [ { options.path = ../../grow-dashboard.json; } ];
    };
  };

}
