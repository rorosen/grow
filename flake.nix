{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    utils,
    crane,
    deploy-rs,
    ...
  }:
    utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
      # pkgs = import nixpkgs { inherit system; };
      craneLib = crane.lib.${system};
      src = craneLib.cleanCargoSource (craneLib.path ./.);

      commonArgs = {
        inherit src;
        inherit (craneLib.crateNameFromCargoToml {cargoToml = ./Cargo.toml;}) version;
        pname = "grow-common";
        doCheck = false;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      agent = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml {cargoToml = ./agent/Cargo.toml;}) pname;
          cargoExtraArgs = "--bin grow-agent";
        });
    in {
      packages = {
        agent = agent;
        service = import ./nix/agent-service.nix inputs;
      };
    })
    // {
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
