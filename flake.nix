{
  description = "Build igneous-md to work on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideScope (
          final: prev: {
            # We override the behavior of `mkCargoDerivation` by adding a wrapper which
            # will set a default value of `CARGO_PROFILE` when not set by the caller.
            # This change will automatically be propagated to any other functions built
            # on top of it (like `buildPackage`, `cargoBuild`, etc.)
            mkCargoDerivation =
              args:
              prev.mkCargoDerivation (
                {
                  CARGO_PROFILE = "dev";
                }
                // args
              );
          }
        );

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt == "js") unfilteredRoot)
            (lib.fileset.maybeMissing ./src)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            openssl
            pkg-config
            webkitgtk_4_1
            noto-fonts-color-emoji
            glib-networking
          ];

          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
            webkitgtk_4_1
            noto-fonts-color-emoji

            wrapGAppsHook4
          ];

        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        igneous-md = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          }
        );

        igneous-md-release = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
            CARGO_PROFILE = "release";
          }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit igneous-md;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          igneous-md-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          igneous-md-test = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          igneous-md-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          # Check formatting
          igneous-md-fmt = craneLib.cargoFmt {
            inherit src;
          };

        };

        packages = {
          default = igneous-md;
          release = igneous-md-release;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = igneous-md;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = commonArgs.nativeBuildInputs;

          buildInputs =
            with pkgs;
            [
              # Add rustup, so that cargo autocomplete works in zsh
              rustup
              rust-bin.stable.latest.default
            ]
            ++ commonArgs.buildInputs;
        };

      }
    );
}
