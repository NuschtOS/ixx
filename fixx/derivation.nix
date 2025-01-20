{ lib, rustPlatform, binaryen, rustc, wasm-pack, wasm-bindgen-cli }:

let
  wasm-bindgen-100 = wasm-bindgen-cli.override {
    version = "0.2.100";
    hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
    cargoHash = "sha256-tD0OY2PounRqsRiFh8Js5nyknQ809ZcHMvCOLrvYHRE=";
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
    wasm-bindgen-100
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
