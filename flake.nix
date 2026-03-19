{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        inherit (pkgs) lib;

        custom-rust-bin = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain custom-rust-bin;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = [];
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "short_signed_id";
            version = "0.0.0";
          });
        individualCrateArgs =
          commonArgs
          // {
            inherit cargoArtifacts;
          };

        fileSetForCrate = crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources ./crates/short_signed_id)
              (craneLib.fileset.commonCargoSources crate)
            ];
          };

        server-app = let
          path = ./crates/server_app;
        in
          craneLib.buildPackage (individualCrateArgs
            // {
              src = fileSetForCrate path;
              inherit (craneLib.crateNameFromCargoToml {src = path;}) pname version;
            });
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            custom-rust-bin
          ];
        };

        packages.default = server-app;
      }
    );
}
