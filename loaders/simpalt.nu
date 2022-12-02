# Main prompt
let-env PROMPT_COMMAND = {
  let args = [((ansi yellow) + '⏾')]

  let args = if $env.LAST_EXIT_CODE == 0 {
    $args
  } else {
    $args | append '-e'
  }

  let args = if $env.SIMPALT_LONG {
    $args | append '-l'
  } else {
    $args
  }

  simpalt l $args
}

# Right prompt
let-env PROMPT_COMMAND_RIGHT = { simpalt r }

# Allow toggling
# Tip: add a keymap calling this command
#
# {
#   name: toggle_simpalt
#   modifier: Control
#   keycode: Char_T
#   mode: [ emacs vi_normal vi_insert ]
#   event: {
#     send: ExecuteHostCommand
#     cmd: 'toggle_simpalt'
#   }
# }
def-env toggle_simpalt [] {
  let-env SIMPALT_LONG = (not $env.SIMPALT_LONG)
  print -n ((ansi -e 'F') + (ansi -e 'J'))
}
let-env SIMPALT_LONG = false
