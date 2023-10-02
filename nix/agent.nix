{
  craneLib,
  stdenv,
  qemu,
  pkg-config,
}: let
  src = craneLib.cleanCargoSource (craneLib.path ./..);

  commonArgs = {
    inherit src;
    inherit (craneLib.crateNameFromCargoToml {cargoToml = ../Cargo.toml;}) version;
    pname = "grow-common";
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (commonArgs
    // {
      depsBuildBuild = [
        qemu
      ];
      nativeBuildInputs = [
        pkg-config
      ];
      inherit cargoArtifacts;
      inherit (craneLib.crateNameFromCargoToml {cargoToml = ../agent/Cargo.toml;}) pname;
      cargoExtraArgs = "--target aarch64-unknown-linux-gnu --bin grow-agent";
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
      HOST_CC = "${stdenv.cc.nativePrefix}cc";
    })
