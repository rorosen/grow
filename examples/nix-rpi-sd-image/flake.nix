{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    nixos-hardware.url = "github:NixOs/nixos-hardware";
    grow = {
      url = "github:rorosen/grow";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, nixpkgs, ... }:
    {
      nixosConfigurations.pi = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = [ ./configuration.nix ];
        specialArgs = {
          inherit inputs;
        };
      };

      packages.aarch64-linux.sdImage = self.nixosConfigurations.pi.config.system.build.sdImage;
    };
}
