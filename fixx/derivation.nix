{ lib, rustPlatform, binaryen, rustc, wasm-pack, wasm-bindgen-cli }:

let
  wasm-bindgen-95 = wasm-bindgen-cli.override {
    version = "0.2.95";
    hash = "sha256-prMIreQeAcbJ8/g3+pMp1Wp9H5u+xLqxRxL+34hICss=";
    cargoHash = "sha256-6iMebkD7FQvixlmghGGIvpdGwFNLfnUcFke/Rg8nPK4=";
  };
in
rustPlatform.buildRustPackage rec {
  pname = "fixx";
  version = "0.1.0";

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    binaryen
    rustc.llvmPackages.lld
    wasm-pack
    wasm-bindgen-95
  ];

  buildPhase = ''
    export HOME=$(mktemp -d)
    (cd fixx && wasm-pack build --release --target web)
  '';

  installPhase = ''
    cp -r fixx/pkg $out
  '';

  cargoBuildFlags = "-p ${pname}";
  cargoTestFlags = "-p ${pname}";
}
