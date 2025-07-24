{ lib
, rustPlatform
, binaryen
, rustc
, wasm-pack
, wasm-bindgen-cli_0_2_100
}:

let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  pname = "fixx";
  inherit (manifest) version;

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    binaryen
    rustc.llvmPackages.lld
    wasm-pack
    wasm-bindgen-cli_0_2_100
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
