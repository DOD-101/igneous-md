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
      self,
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

        src = lib.fileset.toSource rec {
          root = ./.;
          fileset = lib.fileset.unions [
            (lib.fileset.maybeMissing ./crates)
            (lib.fileset.fileFilter (file: file.hasExt == "js") root)
            (lib.fileset.maybeMissing ./crates/igneous-md/src)
            (craneLib.fileset.commonCargoSources root)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            openssl
            pkg-config
            webkitgtk_6_0
            noto-fonts-color-emoji
            glib-networking
          ];

          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
            webkitgtk_6_0
            noto-fonts-color-emoji

            wrapGAppsHook4

            # only needed for testing

            cargo-all-features
          ];

        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources ./crates/igneous-md-viewer)
              (lib.fileset.maybeMissing ./assets)
              crate
            ];
          };

        igneous-md = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "igneous-md";
            cargoExtraArgs = "-p igneous-md";
            src = fileSetForCrate ./crates/igneous-md;
          }
        );

        igneous-md-viewer = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "igneous-md-viewer";
            cargoExtraArgs = "-p igneous-md-viewer";
            src = fileSetForCrate ./crates/igneous-md-viewer;
          }
        );

        igneous-md-release = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "igneous-md";
            cargoExtraArgs = "-p igneous-md";
            src = fileSetForCrate ./crates/igneous-md;

            CARGO_PROFILE = "release";
          }
        );

      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit igneous-md igneous-md-viewer;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          workspace-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          workspace-doc-test = craneLib.cargoDocTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          # Check formatting
          workspace-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # default = workspace-fmt;

        };

        packages = rec {
          inherit igneous-md igneous-md-viewer igneous-md-release;
          default = igneous-md;

          workspace-cargo-test-all-features = craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              buildPhaseCargoCommand = "RUSTFLAGS=\"-D warnings\" cargo test-all-features";
              nativeBuildInputs = commonArgs.nativeBuildInputs;
            }
          );

        };

        apps = rec {
          igneous-md-app = flake-utils.lib.mkApp {
            drv = igneous-md;
          };

          igneous-md-viewer-app = flake-utils.lib.mkApp {
            drv = igneous-md-viewer;
          };

          default = igneous-md-app;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          packages = with pkgs; [
            prek
            typos-lsp
          ];

          shellHook = ''
            prek install
          '';
        };
      }
    );
}
