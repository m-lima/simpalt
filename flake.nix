{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        sharedOptions = {
          systemLinker = pkgs.stdenv.isLinux;
          buildInputs = pkgs: [ pkgs.openssl ];
          nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
          formatters = {
            shfmt.enable = true;
            yamlfmt.enable = true;
          };
        };

        promptOptions = sharedOptions // {
          overrides = {
            checks = {
              glue = pkgs.runCommand "checkglue" { src = ./.; } ''
                ${pkgs.coreutils}/bin/touch $out
                VERSION=$(${pkgs.dasel}/bin/dasel query 'workspace.package.version' -i toml -o yaml < $src/Cargo.toml)
                ZSH=$(${pkgs.gnused}/bin/sed 's/%%VERSION%%/'"$VERSION"'/g' $src/loader/simpalt.zsh)
                NU=$(${pkgs.gnused}/bin/sed 's/%%VERSION%%/'"$VERSION"'/g' $src/loader/simpalt.nu)
                echo Checking version presence
                [ -n "$VERSION" ]
                echo Checking ZSH integration
                ${pkgs.diffutils}/bin/diff $src/simpalt.zsh <(echo "$ZSH")
                echo Checking NU integration
                ${pkgs.diffutils}/bin/diff $src/simpalt.nu <(echo "$NU")
              '';
            };
          };
        };

        tmuxOptions = sharedOptions // {
          buildInputs =
            pkgs:
            [
              pkgs.openssl
            ]
            ++ pkgs.lib.optional pkgs.stdenv.isLinux pkgs.dbus;
          overrides = {
            devShell =
              prev:
              prev
              // {
                env = prev.env // {
                  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.dbus ];
                };
              };
          };
        };

        allOptions =
          promptOptions
          // tmuxOptions
          // {
            buildInputs = pkg: (promptOptions.buildInputs pkg) ++ (tmuxOptions.buildInputs pkg);
            overrides = promptOptions.overrides // tmuxOptions.overrides;
          };

        all = helper.lib.rust.helper inputs system ./. allOptions;
        prompt = helper.lib.rust.helper inputs system ./. (
          promptOptions
          // {
            overrides = {
              mainArgs = {
                pname = "simpalt-prompt";
              };
            };
            cargoExtraArgs = "-p simpalt-prompt";
          }
        );
        tmux = helper.lib.rust.helper inputs system ./. (
          tmuxOptions
          // {
            overrides = {
              mainArgs = {
                pname = "simpalt-tmux";
              };
            };
            cargoExtraArgs = "-p simpalt-tmux";
          }
        );
      in
      all.outputs
      // {
        packages = {
          prompt = prompt.outputs.packages.default;
          tmux = tmux.outputs.packages.default;
        };
        apps = {
          prompt = prompt.outputs.apps.default;
          tmux = tmux.outputs.apps.default;
        };
      }
      // {
        lib.zsh =
          {
            symbol,
            toggleBinding ? null,
            minWidth ? 120,
          }:
          ''
            __simpalt_build_prompt() {
              (( $? != 0 )) && local has_error='-e'
              [ "''${jobstates}" ] && local has_jobs='-j'
          ''
          + (
            if toggleBinding == null then
              ''
                simpalt l -m z -s '${symbol}' $has_error $has_jobs
              ''
            else
              ''
                simpalt l -m z $SIMPALT_MODE -s '${symbol}' $has_error $has_jobs
              ''
          )
          + ''
            }

            __simpalt_build_r_prompt() {
              if (( COLUMNS > ${toString minWidth} )); then
                simpalt r -m z
              fi
            }
          ''
          + (
            if toggleBinding == null then
              ""
            else
              ''
                simpalt_toggle_mode() {
                  [ "$SIMPALT_MODE" ] && unset SIMPALT_MODE || SIMPALT_MODE='-l'
                  zle reset-prompt
                }

                # Allow toggling. E.g.:
                # bindkey '${toggleBinding}' simpalt_toggle_mode
                zle -N simpalt_toggle_mode

                # Simpalt toggle
                bindkey '${toggleBinding}' simpalt_toggle_mode
              ''
          )
          + ''
            # Allow `eval` for the prompt
            setopt promptsubst
            PROMPT='$(__simpalt_build_prompt)'
            RPROMPT='$(__simpalt_build_r_prompt)'

            # Avoid penv from setting the PROMPT
            VIRTUAL_ENV_DISABLE_PROMPT=1
          '';
      }
    );
}
