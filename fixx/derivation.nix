{ lib
, rustPlatform
, binaryen
, nodejs
, rustc
, wasm-pack
, wasm-bindgen-cli_0_2_104
, release ? true
}:

let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  pname = "fixx";
  inherit (manifest) version;

  outputs = [ "out" "dist" ];

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    binaryen
    nodejs # for npm
    rustc.llvmPackages.lld
    wasm-pack
    wasm-bindgen-cli_0_2_104
  ];

  buildPhase = ''
    export HOME=$(mktemp -d)

    cd fixx
    wasm-pack build --${if release then "release" else "dev"} --target web --scope nuschtos
    cd pkg
    npm pack
    cd ../..
  '';

  installPhase = ''
    cp -r fixx/pkg $out
    mkdir $dist
    mv $out/nuschtos-fixx-*-git.tgz $dist/
  '';

  cargoBuildFlags = "-p ${pname}";
  cargoTestFlags = "-p ${pname}";
}
