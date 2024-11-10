{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = {
    cargo2nix,
    flake-utils,
    nixpkgs,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.75.0";
          packageFun = import ./Cargo.nix;
        };
      in rec {
        packages = {
          simpalt = (rustPkgs.workspace.simpalt {});
          default = packages.simpalt;
          zsh = { symbol, toggleBinding ? null }: ''
            __simpalt_build_prompt() {
              (( ? != 0 )) && local has_error='-e'
              [ "''${jobstates}" ] && local has_jobs='-j'
            ''
            + (if toggleBinding == null then ''
              simpalt l -z '${symbol}' $has_error $has_jobs
            '' else ''
              simpalt l -z $SIMPALT_MODE '${symbol}' $has_error $has_jobs
            '')
            + ''
            }

            __simpalt_build_r_prompt() {
              if (( COLUMNS > 120 )); then
                simpalt r -z
              fi
            }
            ''
            + (if toggleBinding == null then '''' else ''
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
        };
      }
    );
}
