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
      dashboards.settings.providers =
        let
          grow-dashboard = pkgs.writeTextDir "grow-dashboard.json" (
            builtins.readFile ../../grow-dashboard.json
          );
        in
        [ { options.path = ../../grow-dashboard.json; } ];
      # dashboards.settings =
      #   let
      #     nodeExporterFull = pkgs.writeTextDir "node-exporter-full.json" (
      #       builtins.readFile (
      #         pkgs.fetchurl {
      #           url = "https://github.com/rfmoz/grafana-dashboards/blob/1e33ce6655776bc6ceeafe202b78c19464889462/prometheus/node-exporter-full.json";
      #           hash = config.nodeExporterHash;
      #         }
      #       )
      #     );
      #
      #     lokiDashboard = pkgs.writeTextDir "loki-dashboard.json" (
      #       builtins.readFile (
      #         pkgs.fetchurl {
      #           url = "https://grafana.com/api/dashboards/13186/revisions/1/download";
      #           hash = config.lokiDashboardHash;
      #         }
      #       )
      #     );
      #   in
      #   {
      #     apiVersion = 1;
      #     providers = [
      #       {
      #         name = "default";
      #         options.path = pkgs.symlinkJoin {
      #           name = "dashboards";
      #           paths = [
      #             nodeExporterFull
      #             lokiDashboard
      #           ];
      #         };
      #       }
      #     ];
      #   };
    };
  };

}
