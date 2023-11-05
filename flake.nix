{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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
      crossSystem = "aarch64-linux";

      pkgs = import nixpkgs {
        inherit crossSystem localSystem;
        overlays = [(import rust-overlay)];
      };

      rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
        targets = ["aarch64-unknown-linux-gnu"];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      agent = pkgs.callPackage ./nix/agent.nix {inherit craneLib;};
    in {
      packages = {
        inherit agent;
        service = import ./nix/agent-service.nix {
          inherit agent;
          inherit (nixpkgs.lib) nixosSystem;
          pkgs = pkgs.pkgsBuildHost;
        };
      };
    })
    // {
      deploy.nodes.growPi = {
        hostname = "192.168.50.102";
        profiles.grow = {
          user = "root";
          sshUser = "rob";
          path = deploy-rs.lib.aarch64-linux.activate.custom self.packages.x86_64-linux.service "./bin/activate";
        };
      };
    };
}
