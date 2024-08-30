{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

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
      crane,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        localSystem = system;
        crossSystem = "aarch64-linux";
        overlays = [ (import rust-overlay) ];
      };

      crossToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
        targets = [ "aarch64-unknown-linux-gnu" ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain (_p: crossToolchain);
      crates = pkgs.callPackage ./nix/build.nix { inherit craneLib; };
    in
    {
      packages.${system} = {
        inherit (crates)
          agent
          server
          gpiotest
          sensortest
          ;
      };

      nixosModules.default = ./nix/module.nix;
      overlays.default = _final: _prev: {
        grow-agent = crates.agent;
        grow-server = crates.server;
        grow-gpiotest = crates.gpiotest;
        grow-sensortest = crates.sensortest;
      };

      # devShells.${system}.default = craneLib.devShell {
      #   packages = with pkgs; [
      #     pkg-config
      #     sqlx-cli
      #   ];
      # };
    };
}
