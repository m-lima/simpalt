if `command -v simpalt &> /dev/null`; then
  if [[ "`simpalt v`" != "0.3.4" ]]; then
    echo '[33mPrompt info:[m Expected version [37m0.3.4[m but `simpalt` is reporting version [37m'`simpalt v`'[m'
    echo 'Check [34mhttps://github.com/m-lima/simpalt-rs/releases[m for the latest version'
  fi

  simpalt_toggle_mode() {
    [ "$SIMPALT_MODE" ] && unset SIMPALT_MODE || SIMPALT_MODE='-l'
    zle reset-prompt
  }

  # Allow toggling. E.g.:
  # bindkey '^T' simpalt_toggle_mode
  zle -N simpalt_toggle_mode

  precmd() {
    if (( ? != 0 )) && local has_error='-e'
    [ "$(jobs)" ] && local has_jobs='-j'
    PROMPT="$(simpalt l -z $SIMPALT_MODE $COMPUTER_SYMBOL $has_error $has_jobs)"

    if (( COLUMNS > 120 )); then
      RPROMPT="$(simpalt r -z)"
    else
      RPROMPT=""
    fi
  }

  # Avoid penv from setting the PROMPT
  VIRTUAL_ENV_DISABLE_PROMPT=1
else
  echo '[31mPrompt error:[m `simpalt` not found. Make sure that it is in your [33m$PATH[m. Reverting to default prompt'
  echo 'Binaries available for major platforms at [34mhttps://github.com/m-lima/simpalt-rs/releases[m'
fi
