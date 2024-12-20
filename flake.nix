{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    { nixpkgs, flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          config,
          pkgs,
          lib,
          system,
          ...
        }:
        let
          nativeBuildInputs =
            with pkgs;
            [
              pkg-config
              rustPlatform.bindgenHook
            ]
            ++ lib.optionals stdenv.isDarwin [ makeBinaryWrapper ];
          buildInputs =
            with pkgs;
            [ openssl ]
            ++ lib.optionals stdenv.isLinux (
              with pkgs;
              [
                libxkbcommon
                vulkan-loader
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
                alsa-lib.dev
                wayland.dev
                udev.dev
              ]
            )
            ++ lib.optionals stdenv.isDarwin (
              with darwin.apple_sdk.frameworks;
              [
                IOKit
                SystemConfiguration
                AppKit
              ]
            );
        in
        {
          packages.default =
            let
              manifest = (lib.importTOML ./Cargo.toml).package;
            in
            pkgs.rustPlatform.buildRustPackage {
              inherit buildInputs nativeBuildInputs;
              inherit (manifest) name version;

              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };
              meta.mainProgram = manifest.name;
            };
          devShells.default = pkgs.mkShell {
            name = "dev-shell";
            inherit nativeBuildInputs;

            buildInputs =
              let
                overlays = [ (import inputs.rust-overlay) ];
                pkgs = import (inputs.nixpkgs) { inherit system overlays; };
              in
              buildInputs
              ++ (with pkgs; [
                gh
                dasel
                cargo-udeps
              ])
              ++ (with pkgs.rust-bin; [
                (stable.latest.minimal.override {
                  extensions = [
                    "clippy"
                    "rust-src"
                  ];
                })

                nightly.latest.clippy
                nightly.latest.rustfmt
                nightly.latest.rust-analyzer
              ]);
          };

        };
    };
}
