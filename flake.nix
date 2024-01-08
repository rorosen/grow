{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    falke-utils.url = "github:numtide/flake-utils";

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
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    deploy-rs,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (localSystem: let
      pkgs = import nixpkgs {
        inherit localSystem;
        overlays = [(import rust-overlay)];
      };
      toolchain = pkgs.rust-bin.stable.latest.default;

      crossPkgs = import nixpkgs {
        inherit localSystem;
        crossSystem = "aarch64-linux";
        overlays = [(import rust-overlay)];
      };

      crossToolchain = crossPkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
        targets = ["aarch64-unknown-linux-gnu"];
      };

      agent = crossPkgs.callPackage ./nix/agent.nix {
        craneLib = (crane.mkLib crossPkgs).overrideToolchain crossToolchain;
      };

      measurement-service = pkgs.callPackage ./nix/measurement-service.nix {
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      };
    in {
      packages = {
        inherit agent measurement-service;
        agent-service = import ./nix/agent-service.nix {
          inherit pkgs agent;
          inherit (nixpkgs.lib) nixosSystem;
        };
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [pkgs.protobuf];
      };
    })
    // {
      deploy.nodes.growPi = {
        hostname = "192.168.50.63";
        profiles.grow = {
          user = "root";
          sshUser = "rob";
          path = deploy-rs.lib.aarch64-linux.activate.custom self.packages.x86_64-linux.service "./bin/activate";
        };
      };
    };
}
