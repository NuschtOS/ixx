{ lib, rustPlatform }:

let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  pname = "ixx";
  inherit (manifest) version;

  src = lib.cleanSource ../.;
  cargoLock = import ../lockfile.nix;

  cargoBuildFlags = "-p ${pname}";
  cargoTestFlags = "-p ${pname}";

  meta = {
    mainProgram = "ixx";
  };
}
