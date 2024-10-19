{
  lib,
  craneLib,
  stdenv,
  python3,
}:
let
  isAarch64 = stdenv.targetPlatform.system == "aarch64-linux";
  isCross = with stdenv; buildPlatform.system != targetPlatform.system;
  assetFilter = path: _type: builtins.match ".*\.pest$|.*\.sql$" path != null;
  sourceFilter = path: type: (assetFilter path type) || (craneLib.filterCargoSources path type);
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  commonArgs =
    {
      src = lib.cleanSourceWith {
        src = craneLib.path ./../..;
        filter = sourceFilter;
      };
      strictDeps = true;
      doCheck = true;
      nativeBuildInputs = [
        python3
      ];
    }
    // (lib.optionalAttrs isAarch64 {
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
      HOST_CC = "${stdenv.cc.nativePrefix}cc";
      TARGET_CC = "${stdenv.cc.targetPrefix}cc";
    })
    // (lib.optionalAttrs isCross {
      PYO3_CROSS_PYTHON_VERSION = python3.pythonVersion;
      RUSTFLAGS = "-L ${python3}/lib";
    });

  crateArgs =
    pname:
    let
      cargoExtraArgs =
        "--package=${pname}" + lib.optionalString isAarch64 " --target aarch64-unknown-linux-gnu";
    in
    {
      inherit
        cargoArtifacts
        cargoExtraArgs
        pname
        ;
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
