# Based on Roby Russel's agnoster theme
# https://github.com/agnoster/agnoster-zsh-theme
# https://github.com/robbyrussell/oh-my-zsh/wiki/themes
#
# # Goals
# - Make a smaller footprint in the termial while maintaning the information
#   provided by agnoster
# - Allow switching between full prompt and small prompt
# - Warn on aws-vault session being active

### Segment drawing
# A few utility functions to make it easy and re-usable to draw segmented prompts

typeset -aHg SIMPALT_PROMPT_SEGMENTS=(
    prompt_aws
    prompt_status
    prompt_context
    prompt_virtualenv
    prompt_dir
    prompt_git
)

typeset -g SIMPALT_SMALL='ON'

CURRENT_BG='NONE'
if [[ -z "$PRIMARY_FG" ]]; then
	PRIMARY_FG=black
fi

# Characters
SEGMENT_SEPARATOR="\ue0b0"
PLUSMINUS="\u00b1"
BRANCH="\ue0a0"
DETACHED="\u27a6"
CROSS="\u2718"
LIGHTNING="\u26a1"
GEAR="\u2699"

# Begin a segment
# Takes two arguments, background and foreground. Both can be omitted,
# rendering default background/foreground.
prompt_segment() {
  local bg fg
  [[ -n $1 ]] && bg="%K{$1}" || bg="%k"
  [[ -n $2 ]] && fg="%F{$2}" || fg="%f"
  if [[ $CURRENT_BG != 'NONE' && $1 != $CURRENT_BG ]]; then
    print -n " %{$bg%F{$CURRENT_BG}%}$SEGMENT_SEPARATOR%{$fg%}"
  else
    print -n "%{$bg%}%{$fg%}"
  fi
  CURRENT_BG=$1
  if [[ -n $3 ]]; then
    print -n " $3"
  fi
}

# End the prompt, closing any open segments
prompt_end() {
  if [[ -n $CURRENT_BG ]]; then
    print -n " %{%k%F{$CURRENT_BG}%}$SEGMENT_SEPARATOR"
  else
    print -n "%{%k%}"
  fi
  print -n "%{%f%}"
  CURRENT_BG=''
}

### Prompt components
# Each component will draw itself, and hide itself if no information needs to be shown

# Context: user@hostname (who am I and where am I)
prompt_context() {
  local context user=`whoami`

  if [[ "$user" != "$DEFAULT_USER" || -n "$SSH_CONNECTION" ]]; then
    [[ -n "${COMPUTER_SYMBOL}" ]] && context="${COMPUTER_SYMBOL}" || context="$user@%m"
    prompt_segment $PRIMARY_FG default "%(!.%{%F{yellow}%}.)$context"
  fi
}

# Git: branch/detached head, dirty status
prompt_git() {
  local ref color
  is_dirty() {
    test -n "$(git status --porcelain --ignore-submodules)"
  }
  ref="$vcs_info_msg_0_"

  if [ $SIMPALT_SMALL ]; then
    if [ -z "$ref" ]; then
      color=blue
    else
      if ! $(git symbolic-ref HEAD &> /dev/null); then
        color=red
      elif is_dirty; then
        color=yellow
      else
        color=green
      fi

      if [[ "${ref}" == "master" ]] || [[ "${ref}" == "develop" ]] || [[ "${ref}" == "development" ]]; then
        ref=""
      else
        ref="${ref/*\//}"
      fi
    fi
    prompt_segment $color $PRIMARY_FG "$ref"
  else
    if [[ -n "$ref" ]]; then
      if is_dirty; then
        color=yellow
        ref="${ref} $PLUSMINUS"
      else
        color=green
        ref="${ref}"
      fi
      if [[ "${ref/.../}" == "$ref" ]]; then
        ref="$BRANCH $ref"
      else
        ref="$DETACHED ${ref/.../}"
      fi
      prompt_segment $color $PRIMARY_FG "${ref}"
    fi
  fi
}

# AWS: current aws-vault session
prompt_aws() {
  if [ $AWS_VAULT ]; then
    [ $SIMPALT_SMALL ] && prompt_segment black default "%{%F{magenta}%}" || prompt_segment magenta $PRIMARY_FG " $AWS_VAULT"
  fi
}

# Dir: current working directory
prompt_dir() {
  if [ $SIMPALT_SMALL ]; then
    if [[ "$PWD" == "$HOME" ]]; then
      prompt_segment black default '~'
    else
      prompt_segment black default "$(basename $PWD)"
    fi
  else
    prompt_segment blue $PRIMARY_FG '%~'
  fi
}

# Status:
# - was there an error
# - am I root
# - are there background jobs?
prompt_status() {
  local symbols
  symbols=()
  [[ $RETVAL -ne 0 ]] && symbols+="%{%F{red}%}$CROSS"
  [[ $UID -eq 0 ]] && symbols+="%{%F{yellow}%}$LIGHTNING"
  [[ $(jobs -l | wc -l) -gt 0 ]] && symbols+="%{%F{cyan}%}$GEAR"

  [[ -n "$symbols" ]] && prompt_segment $PRIMARY_FG default "$symbols"
}

# Display current virtual environment
prompt_virtualenv() {
  if [[ -n $VIRTUAL_ENV ]]; then
    color=cyan
    prompt_segment $color $PRIMARY_FG "$(basename $VIRTUAL_ENV)"
  fi
}

## Main prompt
prompt_simpalt_main() {
  RETVAL=$?
  CURRENT_BG='NONE'
  for prompt_segment in "${SIMPALT_PROMPT_SEGMENTS[@]}"; do
    [[ -n $prompt_segment ]] && $prompt_segment
  done
  prompt_end
}

prompt_simpalt_precmd() {
  vcs_info
  PROMPT='%{%f%b%k%}$(prompt_simpalt_main) '
}

prompt_simpalt_setup() {
  autoload -Uz add-zsh-hook
  autoload -Uz vcs_info

  prompt_opts=(cr subst percent)

  add-zsh-hook precmd prompt_simpalt_precmd

  zstyle ':vcs_info:*' enable git
  zstyle ':vcs_info:*' check-for-changes false
  zstyle ':vcs_info:git*' formats '%b'
  zstyle ':vcs_info:git*' actionformats '%b (%a)'
}

prompt_simpalt_setup "$@"
