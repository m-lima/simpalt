if `command -v simpalt &> /dev/null`; then
  if [[ "`simpalt v`" != "0.2.0" ]]; then
    echo '[33mPrompt info:[m Expected version [37m0.2.0[m but `simpalt` is reporting version [37m'`simpalt v`'[m'
    echo 'Check [34mhttps://github.com/m-lima/simpalt-rs/releases[m for the latest version'
  fi

  __simpalt_build_prompt() {
    (( ? > 0 )) && error='-e'
    [ "$(jobs)" ] && jobs='-j'
    simpalt l $SIMPALT_MODE $COMPUTER_SYMBOL $error $jobs | simpalt c zsh
  }

  __simpalt_build_r_prompt() {
    if (( COLUMNS > 120 )); then
      simpalt r | simpalt c zsh
    fi
  }

  simpalt_toggle_mode() {
    [ "$SIMPALT_MODE" ] && unset SIMPALT_MODE || SIMPALT_MODE='-l'
    zle reset-prompt
  }

  # Allow toggling. E.g.:
  # bindkey '^T' simpalt_toggle_mode
  zle -N simpalt_toggle_mode

  # Allow `eval` for the prompt
  setopt promptsubst
  PROMPT='$(__simpalt_build_prompt)'
  RPROMPT='$(__simpalt_build_r_prompt)'

  # Avoid penv from setting the PROMPT
  VIRTUAL_ENV_DISABLE_PROMPT=1
else
  echo '[31mPrompt error:[m `simpalt` not found. Make sure that it is in your [33m$PATH[m. Reverting to default prompt'
  echo 'Binaries available for major platforms at [34mhttps://github.com/m-lima/simpalt-rs/releases[m'
fi
