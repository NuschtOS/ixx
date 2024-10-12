{ lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "ixx";
  version = "0.1.0";

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = "-p ${pname}";
  cargoTestFlags = "-p ${pname}";
}
