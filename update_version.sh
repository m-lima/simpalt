#!/usr/bin/env bash

version="$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "simpalt") | .version')"

if [[ "${version}" ]]; then
  sed 's/%%VERSION%%/'"${version}"'/g' loader/simpalt.zsh > simpalt.zsh
  sed 's/%%VERSION%%/'"${version}"'/g' loader/simpalt.nu > simpalt.nu
else
  echo "[31mERROR[m Could not fetch simpalt version" >&2
  exit 1
fi
