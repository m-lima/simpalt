Simpalt
---------------------

[![Github](https://github.com/m-lima/simpalt/actions/workflows/build.yml/badge.svg)](https://github.com/m-lima/simpalt/actions/workflows/build.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A blazing fast ZSH and NuShell theme written in Rust with focus on information density, screen real estate, and beauty.

###### Demo
![Demo](.github/res/demo.gif)

How to use it
---------------------

### Software Requirement
[ZSH](https://www.zsh.org/) or [NuShell](https://www.nushell.sh/)

### Suggested setup

Using NixOS, add the following to your system flake:
```nix
{
  inputs = {
    simpalt.url = "github:m-lima/simpalt";
  };

  outputs =
    {
      ...
    }:
    {
      [...]
      home-manager =
        let
          simpalt = {
            pkg = inputs.simpalt.packages.${pkgs.stdenv.hostPlatform.system}.default;
            zsh = inputs.simpalt.lib.zsh;
          };
        in:
        {
          home.packages = [ simpalt.pkg ];
          programs.zsh.initContent = simpalt.zsh {
            symbol = "₵";
            toggleBinding = "^T";
          };
        };
    };
}
```

### Loading directly

* Get the binary by either:
    * Downloading from the [release page](https://github.com/m-lima/simpalt/releases)
    * Copiling with Rust
* Load the [`simpalt.zsh`](/simpalt.zsh) or [`simpalt.nu`](/simpalt.nu) in your initialization script
