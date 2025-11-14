{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };
          inherit (pkgs) lib;
        in
        {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              pnpm
              rustc
              rustc.llvmPackages.lld
              wasm-pack
            ];

            RUST_SRC_PATH = pkgs.rust.packages.stable.rustPlatform.rustLibSrc;
          };

          packages = let
            rustPlatform = if lib.versionAtLeast pkgs.rustc.version "1.88" then
              pkgs.rustPlatform
            else
              pkgs.rustPackages_1_89.rustPlatform;
          in {
            ixx = pkgs.callPackage ./ixx/derivation.nix { inherit rustPlatform; };
            fixx = pkgs.callPackage ./fixx/derivation.nix { inherit rustPlatform; };
            libixx = pkgs.callPackage ./libixx/derivation.nix { inherit rustPlatform; };
          };
        }
      );
}
