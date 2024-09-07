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
    enable = lib.mkEnableOption "the grow Grafana instance";
    provision.enable = (lib.mkEnableOption "the provisioning of Grafana") // {
      default = true;
    };
  };

  config.services.grafana = lib.mkIf cfg.enable {
    enable = true;
    declarativePlugins = lib.mkIf cfg.provision.enable [
      pkgs.grafanaPlugins.yesoreyeram-infinity-datasource
    ];
    settings.server = {
      protocol = lib.mkDefault "http";
      http_addr = lib.mkDefault "::";
      http_port = lib.mkDefault 80;
    };

    provision = {
      enable = cfg.provision.enable;
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
