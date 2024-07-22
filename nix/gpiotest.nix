{ craneLib, stdenv }:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource (craneLib.path ./..);
  strictDeps = true;
  doCheck = false;
  inherit (craneLib.crateNameFromCargoToml { cargoToml = ../gpiotest/Cargo.toml; }) pname;
  cargoExtraArgs = "--target aarch64-unknown-linux-gnu --package=grow-gpiotest";
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
}
