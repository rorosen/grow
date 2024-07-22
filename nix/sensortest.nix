{ craneLib, stdenv }:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource (craneLib.path ./..);
  strictDeps = true;
  doCheck = false;
  inherit (craneLib.crateNameFromCargoToml { cargoToml = ../sensortest/Cargo.toml; }) pname;
  cargoExtraArgs = "--target aarch64-unknown-linux-gnu --package=grow-sensortest";
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
}
