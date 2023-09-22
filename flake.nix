{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    advisory-db,
    deploy-rs,
    ...
  }: let
    system = "aarch64-linux";
    craneLib = crane.lib.${system};
    src = craneLib.cleanCargoSource (craneLib.path ./.);

    commonArgs = {
      inherit src;
    };

    cargoArtifacts = craneLib.buildDepsOnly commonArgs;

    grow = craneLib.buildPackage (commonArgs
      // {
        inherit cargoArtifacts;
      });
  in {
    packages.${system} = {
      default = grow;
      service = import ./nix/grow.nix inputs;
    };

    deploy.nodes.growPi = {
      hostname = "192.168.50.102";
      profiles.grow = {
        user = "root";
        sshUser = "rob";
        path = deploy-rs.lib.aarch64-linux.activate.custom self.packages.aarch64-linux.service "./bin/activate";
      };
    };
  };
}
