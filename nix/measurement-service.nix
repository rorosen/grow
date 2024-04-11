{
  lib,
  craneLib,
}: let
  sqlFilter = path: _: builtins.match ".*\.sql$" path != null;
  sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);
in
  craneLib.buildPackage {
    src = lib.cleanSourceWith {
      src = craneLib.path ./..;
      filter = sqlOrCargo;
    };

    strictDeps = true;
    doCheck = false;
    inherit (craneLib.crateNameFromCargoToml {cargoToml = ../measurement-service/Cargo.toml;}) pname;
    cargoExtraArgs = "--package=measurement-service";
  }
