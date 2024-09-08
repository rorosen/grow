{ pkgs, crane }:
let
  mkCrossToolchain =
    pkgs:
    pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
      targets = [ "aarch64-unknown-linux-gnu" ];
    };

  mkCraneLib = pkgs: (crane.mkLib pkgs).overrideToolchain (_p: (mkCrossToolchain pkgs));
  crates = pkgs.callPackage ./crates.nix { craneLib = mkCraneLib pkgs; };
  yesoreyeram-infinity-datasource = pkgs.callPackage ./yesoreyeram-infinity-datasource.nix { };
in
{
  inherit (crates) agent server sensortest;
  inherit yesoreyeram-infinity-datasource;
}
