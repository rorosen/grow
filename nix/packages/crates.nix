{
  lib,
  craneLib,
  stdenv,
}:
let
  isAarch64 = stdenv.targetPlatform.system == "aarch64-linux";
  sqlFilter = path: _type: builtins.match ".*\.sql$" path != null;
  sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  commonArgs =
    {
      src = lib.cleanSourceWith {
        src = craneLib.path ./../..;
        filter = sqlOrCargo;
      };
      strictDeps = true;
      doCheck = true;
    }
    // (lib.optionalAttrs isAarch64 {
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
      HOST_CC = "${stdenv.cc.nativePrefix}cc";
      TARGET_CC = "${stdenv.cc.targetPrefix}cc";
    });

  crateArgs =
    pname:
    let
      cargoExtraArgs =
        "--package=${pname}" + lib.optionalString isAarch64 " --target aarch64-unknown-linux-gnu";
    in
    {
      inherit pname cargoArtifacts cargoExtraArgs;
    };

  agent = craneLib.buildPackage (
    (crateArgs (craneLib.crateNameFromCargoToml { cargoToml = ../../agent/Cargo.toml; }).pname)
    // commonArgs
  );

  server = craneLib.buildPackage (
    (crateArgs (craneLib.crateNameFromCargoToml { cargoToml = ../../server/Cargo.toml; }).pname)
    // commonArgs
  );

  sensortest = craneLib.buildPackage (
    (crateArgs (craneLib.crateNameFromCargoToml { cargoToml = ../../sensortest/Cargo.toml; }).pname)
    // commonArgs
  );
in
{
  inherit agent server sensortest;
}
