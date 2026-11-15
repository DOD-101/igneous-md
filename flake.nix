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
        packages' = {
          igneous-md = {
            completionsCmd = "completions";
          };
          igneous-md-viewer = {
            completionsCmd = "--completions";
          };
        };

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
          version = (fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;

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
            installShellFiles
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
          name:
          { completionsCmd }:
          release:
          naersk'.buildPackage (
            commonArgs
            // {
              inherit name release;
              cargoBuildOptions =
                old:
                old
                ++ [
                  "-p"
                  name
                ];

              postInstall = lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
                installShellCompletion --cmd ${name} \
                  --bash <($out/bin/${name} ${completionsCmd} bash) \
                  --fish <($out/bin/${name} ${completionsCmd} fish) \
                  --zsh  <($out/bin/${name} ${completionsCmd} zsh)
              '';
            }
          );

        # contains debug and release variants for each package
        workspacePackages =
          let
            variants = lib.mapAttrsToList (name: val: [
              (lib.nameValuePair name (buildWorkspacePackage name val false))
              (lib.nameValuePair "${name}-release" (buildWorkspacePackage name val true))
            ]) packages';
          in
          lib.listToAttrs (lib.flatten variants);

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
          inherit (workspacePackages)
            igneous-md
            igneous-md-viewer
            ;
          inherit
            workspace-clippy
            workspace-doc
            workspace-doc-test
            workspace-fmt
            workspace-cargo-test-all-features
            ;
        };

        packages = workspacePackages // {
          default = workspacePackages.igneous-md;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.checks.${system}.workspace-clippy ];
          packages = with pkgs; [
            cargo
            rustc
            prek
            curl

            typos-lsp
            typescript-language-server
            typescript
          ];

          shellHook = ''
            prek install
          '';
        };
      }
    );
}
