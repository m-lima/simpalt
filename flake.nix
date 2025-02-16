{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ] ++ lib.optionals stdenv.isDarwin [ libiconv ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        simpalt = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            env = {
              CARGO_PROFILE = "mega";
              CARGO_BUILD_RUSTFLAGS = "-C target-cpu=native -C prefer-dynamic=no";
            };
          }
        );

        hack =
          {
            args,
            tools ? [ ],
          }:
          craneLib.mkCargoDerivation (
            commonArgs
            // {
              inherit cargoArtifacts;
              pnameSuffix = "-hack";
              buildPhaseCargoCommand = "cargo hack --feature-powerset --workspace ${args}";
              nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [ pkgs.cargo-hack ] ++ tools;
            }
          );
      in
      {
        checks = {
          inherit simpalt;

          hackCheck = hack {
            args = "check";
          };
          hackCheckTests = hack {
            args = "check --tests";
          };
          hackCheckExamples = hack {
            args = "check --examples";
          };
          hackClippy = hack {
            args = "clippy";
            tools = [ pkgs.clippy ];
          };
          hackClippyTests = hack {
            args = "clippy --tests";
            tools = [ pkgs.clippy ];
          };
          hackClippyExamples = hack {
            args = "clippy --examples";
            tools = [ pkgs.clippy ];
          };
          hackTest = hack {
            args = "test";
          };
        };

        packages.default = simpalt;

        lib.zsh =
          {
            symbol,
            toggleBinding ? null,
          }:
          ''
            __simpalt_build_prompt() {
              (( $? != 0 )) && local has_error='-e'
              [ "''${jobstates}" ] && local has_jobs='-j'
          ''
          + (
            if toggleBinding == null then
              ''
                simpalt l -z '${symbol}' $has_error $has_jobs
              ''
            else
              ''
                simpalt l -z $SIMPALT_MODE '${symbol}' $has_error $has_jobs
              ''
          )
          + ''
            }

            __simpalt_build_r_prompt() {
              if (( COLUMNS > 120 )); then
                simpalt r -z
              fi
            }
          ''
          + (pkgs.lib.optionalString (toggleBinding != null) ''
            simpalt_toggle_mode() {
              [ "$SIMPALT_MODE" ] && unset SIMPALT_MODE || SIMPALT_MODE='-l'
              zle reset-prompt
            }

            # Allow toggling. E.g.:
            # bindkey '${toggleBinding}' simpalt_toggle_mode
            zle -N simpalt_toggle_mode

            # Simpalt toggle
            bindkey '${toggleBinding}' simpalt_toggle_mode
          '')
          + ''
            # Allow `eval` for the prompt
            setopt promptsubst
            PROMPT='$(__simpalt_build_prompt)'
            RPROMPT='$(__simpalt_build_r_prompt)'

            # Avoid penv from setting the PROMPT
            VIRTUAL_ENV_DISABLE_PROMPT=1
          '';

        apps.default = flake-utils.lib.mkApp {
          drv = simpalt;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            cargo-hack
            (pkgs.writeShellScriptBin "cargo-all" ''
              #!/usr/bin/env bash
              shift

              while (( $# > 0 )); do
                case "$1" in
                  nightly)
                    nightly='+nightly' ;;
                  run|r)
                    run=1 ;;
                  clean|c)
                    clean=1 ;;
                esac
                shift
              done

              if [ $clean ]; then
                echo "[34mCleaning[m" && \
                cargo clean
              fi && \
              echo "[34mFormatting[m" && \
              cargo $nightly fmt --all && \
              echo "[34mChecking main[m" && \
              cargo $nightly hack --feature-powerset check --workspace $@ && \
              echo "[34mChecking examples[m" && \
              cargo $nightly hack --feature-powerset check --workspace --examples $@ && \
              echo "[34mChecking tests[m" && \
              cargo $nightly hack --feature-powerset check --workspace --tests $@ && \
              echo "[34mLinting main[m" && \
              cargo $nightly hack --feature-powerset clippy --workspace $@ && \
              echo "[34mLinting tests[m" && \
              cargo $nightly hack --feature-powerset clippy --workspace --tests $@ && \
              echo "[34mLinting examples[m" && \
              cargo $nightly hack --feature-powerset clippy --workspace --examples $@ && \
              echo "[34mTesting main[m" && \
              cargo $nightly hack --feature-powerset test --workspace $@ && \
              if [ "$run" ]; then
                echo "[34mRunning[m" && \
                cargo $nightly run $@
              fi
            '')
          ];
        };

        formatter =
          (treefmt-nix.lib.evalModule pkgs {
            projectRootFile = "Cargo.toml";
            programs = {
              nixfmt.enable = true;
              # nufmt.enable = true;
              rustfmt.enable = true;
              shfmt.enable = true;
              taplo.enable = true;
              yamlfmt.enable = true;
            };
            settings = {
              excludes = [
                "*.lock"
                ".direnv/*"
                ".envrc"
                ".gitignore"
                "result*/"
                "target/*"
              ];
              formatter = {
                shfmt.includes = [
                  "*.zsh"
                ];
              };
            };
          }).config.build.wrapper;
      }
    );
}
