{
  lib,
  craneLib,
  stdenv,
}:
let
  sqlFilter = path: _type: builtins.match ".*\.sql$" path != null;
  sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);
in
craneLib.buildPackage {
  src = lib.cleanSourceWith {
    src = craneLib.path ./..;
    filter = sqlOrCargo;
    name = "source";
  };
  strictDeps = true;
  doCheck = false;
  inherit (craneLib.crateNameFromCargoToml { cargoToml = ../agent/Cargo.toml; }) pname;
  cargoExtraArgs = "--target aarch64-unknown-linux-gnu --package=grow-agent";
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
  HOST_CC = "${stdenv.cc.nativePrefix}cc";
  TARGET_CC = "${stdenv.cc.targetPrefix}cc";
}
