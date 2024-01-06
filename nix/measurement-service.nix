{craneLib}: let
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
      inherit cargoArtifacts;
      inherit (craneLib.crateNameFromCargoToml {cargoToml = ../measurement-service/Cargo.toml;}) pname;
      cargoExtraArgs = "--bin measurement-service";
    })
