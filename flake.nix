{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          localSystem = system;
          crossSystem = "aarch64-linux";
          overlays = [ (import rust-overlay) ];
        };
        mkCrossToolchain =
          pkgs:
          pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
            targets = [ "aarch64-unknown-linux-gnu" ];
          };

        mkCraneLib = pkgs: (crane.mkLib pkgs).overrideToolchain (_p: (mkCrossToolchain pkgs));
        crates = pkgs.callPackage ./nix/packages/crates.nix { craneLib = mkCraneLib pkgs; };
        yesoreyeram-infinity-datasource =
          pkgs.callPackage ./nix/packages/yesoreyeram-infinity-datasource.nix
            { };
      in
      {
        packages = {
          inherit (crates) agent server sensortest;
          inherit yesoreyeram-infinity-datasource;
        };
      }
    )
    // {
      nixosModules = import ./nix/modules;
    };
}
