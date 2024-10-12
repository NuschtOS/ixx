{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };
        in
        {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              rustc.llvmPackages.lld
              wasm-pack
            ];

            RUST_SRC_PATH = pkgs.rust.packages.stable.rustPlatform.rustLibSrc;
          };

          packages = {
            ixx = pkgs.callPackage ./ixx/derivation.nix { };
            fixx = pkgs.callPackage ./fixx/derivation.nix { };
          };
        }
      );
}
