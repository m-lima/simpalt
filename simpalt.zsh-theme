if `command -v simpalt &> /dev/null`; then
  __simpalt_build_prompt() {
    (( ? > 0 )) && error='-e'
    [ "$(jobs)" ] && jobs='-j'
    simpalt e "$(simpalt l $SIMPALT_MODE $COMPUTER_SYMBOL $error $jobs)"
  }

  __simpalt_build_r_prompt() {
    if (( COLUMNS > 120 )); then
      simpalt e "$(simpalt r)"
    fi
  }

  simpalt_toggle_mode() {
    [ "$SIMPALT_MODE" ] && unset SIMPALT_MODE || SIMPALT_MODE='-l'
    zle reset-prompt
  }

  # Allow toggling
  zle -N simpalt_toggle_mode simpalt_toggle_mode

  # Allow `eval` for the prompt
  setopt promptsubst
  PROMPT='$(__simpalt_build_prompt)'
  RPROMPT='$(__simpalt_build_r_prompt)'
else
  echo '[31mPrompt error:[m `simpalt` not found. Make sure that it is in your [33m$PATH[m. Reverting to default prompt'
fi
