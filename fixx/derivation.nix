{ lib, rustPlatform, binaryen, rustc, wasm-pack, wasm-bindgen-cli }:

let
  wasm-bindgen-95 = wasm-bindgen-cli.override {
    version = "0.2.95";
    hash = "sha256-prMIreQeAcbJ8/g3+pMp1Wp9H5u+xLqxRxL+34hICss=";
    cargoHash = "sha256-6iMebkD7FQvixlmghGGIvpdGwFNLfnUcFke/Rg8nPK4=";
  };
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  pname = "fixx";
  inherit (manifest) version;

  src = lib.cleanSource ../.;
  cargoLock = import ../lockfile.nix;

  nativeBuildInputs = [
    binaryen
    rustc.llvmPackages.lld
    wasm-pack
    wasm-bindgen-95
  ];

  buildPhase = ''
    export HOME=$(mktemp -d)
    (cd fixx && wasm-pack build --release --target web --scope nuschtos)
  '';

  installPhase = ''
    cp -r fixx/pkg $out
  '';

  cargoBuildFlags = "-p ${pname}";
  cargoTestFlags = "-p ${pname}";
}
