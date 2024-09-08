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
      inputs.nixpkgs.follows = "nixpkgs";
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
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        pkgsAarch64 = import nixpkgs {
          localSystem = system;
          crossSystem = "aarch64-linux";
          overlays = [ (import rust-overlay) ];
        };
        growPkgs = {
          inherit (pkgs.callPackage ./nix/packages { inherit crane; })
            agent
            server
            sensortest
            yesoreyeram-infinity-datasource
            ;
        };
        growPkgsAarch64 = {
          inherit (pkgsAarch64.callPackage ./nix/packages { inherit crane; })
            agent
            server
            sensortest
            yesoreyeram-infinity-datasource
            ;
        };
      in
      {
        packages = growPkgs // (pkgs.lib.concatMapAttrs (n: v: { "${n}-aarch64" = v; }) growPkgsAarch64);
      }
    )
    // {
      nixosModules = import ./nix/modules;
      overlays.default =
        _final: prev:
        let
          pkgs = prev.extend (import rust-overlay);
          packages = pkgs.callPackage ./nix/packages { inherit crane; };
        in
        {
          grow-agent = packages.agent;
          grow-server = packages.server;
          grow-sensortest = packages.sensortest;
          grafanaPlugins = prev.grafanaPlugins // {
            inherit (packages) yesoreyeram-infinity-datasource;
          };
        };
    };
}
