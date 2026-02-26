{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
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
      in
      (helper.lib.rust.helper inputs system ./. {
        systemLinker = pkgs.stdenv.isLinux;
        buildInputs = pkgs: [ pkgs.openssl ];
        nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
        formatters = {
          shfmt.enable = true;
          yamlfmt.enable = true;
        };
        overrides = {
          checks = {
            glue = pkgs.runCommand "checkglue" { src = ./.; } ''
              ${pkgs.coreutils}/bin/touch $out
              VERSION=$(${pkgs.dasel}/bin/dasel -f $src/Cargo.toml -r toml '.package.version' | ${pkgs.coreutils}/bin/tr -d "'")
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
      }).outputs
    )
    // {
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
    };
}
