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
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      crossPkgs = import nixpkgs {
        localSystem = system;
        crossSystem = "aarch64-linux";
        overlays = [ (import rust-overlay) ];
      };

      crossToolchain = crossPkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
        targets = [ "aarch64-unknown-linux-gnu" ];
      };

      craneLib = (crane.mkLib crossPkgs).overrideToolchain (_p: crossToolchain);
      sampler = crossPkgs.callPackage ./nix/sampler.nix { inherit craneLib; };
      agent = crossPkgs.callPackage ./nix/agent.nix { inherit craneLib; };
      sensortest = crossPkgs.callPackage ./nix/sensortest.nix { inherit craneLib; };
      gpiotest = crossPkgs.callPackage ./nix/gpiotest.nix { inherit craneLib; };
    in
    {
      packages.${system} = {
        inherit
          sampler
          agent
          sensortest
          gpiotest
          ;
        agent-service = import ./nix/agent-service.nix {
          inherit pkgs agent;
          inherit (nixpkgs.lib) nixosSystem;
        };
        install-sampler = import ./nix/install-sampler.nix { inherit pkgs sampler; };
      };

      nixosModules.agent = ./nix/agent-module.nix;
      overlays.default = _final: _prev: {
        grow-agent = agent;
        grow-sensortest = sensortest;
        grow-gpiotest = gpiotest;
      };

      # devShells.${system}.default = craneLib.devShell {
      #   packages = with pkgs; [
      #     pkg-config
      #     sqlx-cli
      #   ];
      # };
    };
}
