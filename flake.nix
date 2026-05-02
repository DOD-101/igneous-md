{
  description = "Build igneous-md to work on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        naersk' = pkgs.callPackage naersk { };

        parseVendorEnv =
          file:
          let
            content = builtins.readFile file;
            lines = lib.splitString "\n" content;
            nonEmpty = builtins.filter (l: l != "" && !(lib.hasPrefix "#" l)) lines;
            pairs = map (
              l:
              let
                parts = lib.splitString "=" l;
              in
              {
                name = builtins.elemAt parts 0;
                value = builtins.elemAt parts 1;
              }
            ) nonEmpty;
          in
          builtins.listToAttrs pairs;

        vendorDeps = parseVendorEnv ./vendor.env;

        vendoredHighlightJs = pkgs.fetchurl {
          url = vendorDeps.HIGHLIGHT_JS_URL;
          hash = vendorDeps.HIGHLIGHT_JS_HASH;
        };

        vendoredMathjaxJs = pkgs.fetchurl {
          url = vendorDeps.MATHJAX_URL;
          hash = vendorDeps.MATHJAX_HASH;
        };

        commonArgs = {
          src = ./.;

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
            esbuild
            git
            wrapGAppsHook4
          ];

          IGNEOUS_VENDOR_ONLY = "1";
          GIT_VERSION_COMMIT = self.rev or "dirty-nix-build";

          preConfigure = ''
            cp ${vendoredHighlightJs} crates/igneous-md-viewer/src/highlight.min.js
            cp ${vendoredMathjaxJs} crates/igneous-md-viewer/src/tex-mml-svg.js
            cp ${./vendor.env} vendor.env
          '';
        };

        buildWorkspacePackage =
          pname: release:
          naersk'.buildPackage (
            commonArgs
            // {
              pname = pname;
              inherit release;
              cargoBuildOptions =
                old:
                old
                ++ [
                  "-p"
                  pname
                ];
            }
          );

        igneous-md = buildWorkspacePackage "igneous-md" false;
        igneous-md-release = buildWorkspacePackage "igneous-md" true;

        igneous-md-viewer = buildWorkspacePackage "igneous-md-viewer" false;
        igneous-md-viewer-release = buildWorkspacePackage "igneous-md-viewer" true;

        buildCheck =
          name: mode: cargoExtraArgs: release:
          naersk'.buildPackage (
            commonArgs
            // {
              inherit mode release;
              pname = name;
              cargoBuildOptions = old: old ++ cargoExtraArgs;
            }
          );

        workspace-clippy = naersk'.buildPackage (
          commonArgs
          // {
            pname = "workspace-clippy";
            mode = "clippy";
            cargoBuildOptions =
              old:
              old
              ++ [
                "--all-targets"
              ];
          }
        );

        workspace-doc = naersk'.buildPackage (
          commonArgs
          // {
            pname = "workspace-doc";
            mode = "build";
            doDoc = true;
            copyDocsToSeparateOutput = true;
          }
        );

        workspace-doc-test = buildCheck "workspace-doc-test" "test" [ "--doc" ] false;

        workspace-fmt = naersk'.buildPackage (
          commonArgs
          // {
            pname = "workspace-fmt";
            mode = "fmt";
            nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.rustfmt ];
          }
        );

        workspace-cargo-test-all-features = buildCheck "workspace-test-all-features" "test" [
          "--all-features"
        ] false;

      in
      {
        checks = {
          inherit
            igneous-md
            igneous-md-viewer
            workspace-clippy
            workspace-doc
            workspace-doc-test
            workspace-fmt
            workspace-cargo-test-all-features
            ;
        };

        packages = rec {
          inherit
            igneous-md
            igneous-md-release
            igneous-md-viewer
            igneous-md-viewer-release
            ;
          default = igneous-md;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.checks.${system}.workspace-clippy ];
          packages = with pkgs; [
            cargo
            rustc
            prek
            typos-lsp
            curl
          ];

          shellHook = ''
            prek install
          '';
        };
      }
    );
}
