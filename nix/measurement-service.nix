{craneLib}:
craneLib.buildPackage {
  src = craneLib.cleanCargoSource (craneLib.path ./..);
  strictDeps = true;
  doCheck = false;
  inherit (craneLib.crateNameFromCargoToml {cargoToml = ../measurement-service/Cargo.toml;}) pname;
  cargoExtraArgs = "--package=measurement-service";
}
