{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
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

          CARGO_BUILD_RUSTFLAGS = "-C target-cpu=native -C prefer-dynamic=no";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        simpalt = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

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

        integrations.zsh =
          {
            symbol,
            toggleBinding ? null,
          }:
          ''
            __simpalt_build_prompt() {
              (( ? != 0 )) && local has_error='-e'
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
          packages = with pkgs; [ cargo-hack ];
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
