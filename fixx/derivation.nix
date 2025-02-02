{ lib
, buildWasmBindgenCli
, rustPlatform
, binaryen
, fetchCrate
, rustc
, wasm-pack
}:

let
  wasm-bindgen-100 = buildWasmBindgenCli rec {
    src = fetchCrate {
      pname = "wasm-bindgen-cli";
      version = "0.2.100";
      hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
    };

    cargoDeps = rustPlatform.fetchCargoVendor {
      inherit src;
      inherit (src) pname version;
      hash = "sha256-qsO12332HSjWCVKtf1cUePWWb9IdYUmT+8OPj/XP2WE=";
    };
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
