{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    deploy-rs = {
      url = "github:serokell/deploy-rs";
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
    { self
    , nixpkgs
    , crane
    , deploy-rs
    , rust-overlay
    , ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      toolchain = pkgs.rust-bin.stable.latest.default;

      crossPkgs = import nixpkgs {
        localSystem = system;
        crossSystem = "aarch64-linux";
        overlays = [ (import rust-overlay) ];
      };

      crossToolchain = crossPkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
        targets = [ "aarch64-unknown-linux-gnu" ];
      };

      sampler = crossPkgs.callPackage ./nix/sampler.nix {
        craneLib = (crane.mkLib crossPkgs).overrideToolchain crossToolchain;
      };

      agent = crossPkgs.callPackage ./nix/agent.nix {
        craneLib = (crane.mkLib crossPkgs).overrideToolchain crossToolchain;
      };

      measurement-service = pkgs.callPackage ./nix/measurement-service.nix {
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      };
    in
    {
      packages.${system} = {
        inherit sampler agent measurement-service;
        agent-service = import ./nix/agent-service.nix {
          inherit pkgs agent;
          inherit (nixpkgs.lib) nixosSystem;
        };
        install-sampler = import ./nix/install-sampler.nix {
          inherit pkgs sampler;
        };
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.protobuf ];
      };

      overlays.default = final: prev: {
        inherit agent measurement-service;
      };

      deploy.nodes.growPi = {
        hostname = "192.168.50.63";
        profiles.grow = {
          user = "root";
          sshUser = "rob";
          # path = deploy-rs.lib.aarch64-linux.activate.custom self.packages.${system}.agent-service "./bin/activate";
          path = deploy-rs.lib.aarch64-linux.activate.custom self.packages.${system}.install-sampler "./bin/activate-sampler";
        };
      };
    };
}
