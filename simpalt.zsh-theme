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

if [[ -z $SIMPALT_PROMPT_SEGMENTS ]]; then
  typeset -aHg SIMPALT_PROMPT_SEGMENTS=(
      prompt_aws
      prompt_status
      prompt_context
      prompt_virtualenv
      prompt_dir
      prompt_git
  )
fi

if [[ -z $SIMPALT_MAIN_BRANCHES ]]; then
  typeset -aHg SIMPALT_MAIN_BRANCHES=(
      master
      development
  )
fi

typeset -g SIMPALT_SMALL='ON'

if [[ -z "$PRIMARY_FG" ]]; then
  PRIMARY_FG=black
fi

# Begin a segment
# Takes two arguments, background and foreground. Both can be omitted,
# rendering default background/foreground.
prompt_segment() {
  local bg fg

  [[ -n $1 ]] && bg="%K{$1}" || bg="%k"
  [[ -n $2 ]] && fg="%F{$2}" || fg="%f"

  if [[ $__SIMPALT_CURRENT_BG == 'NONE' ]]; then
    print -n "%{$bg%}"
  else
    if [[ $1 != $__SIMPALT_CURRENT_BG ]]; then
      if [[ -z $__SIMPALT_PENDING_FLAG ]]; then
        print -n ' '
      fi
      print -n "%{$bg%F{$__SIMPALT_CURRENT_BG}%}"
    elif [[ -n $__SIMPALT_PENDING_FLAG || -z $3 ]]; then
      print -n "%{$fg%}"
    fi
  fi

  print -n "%{$fg%}"

  __SIMPALT_CURRENT_BG=$1

  if [[ -n $3 ]]; then
    print -n " $3"
    unset __SIMPALT_PENDING_FLAG
  else
    __SIMPALT_PENDING_FLAG=1
  fi
}

# End the prompt, closing any open segments
prompt_end() {
  if [[ -z $__SIMPALT_PENDING_FLAG ]]; then
    print -n ' '
  fi

  if [[ -n $__SIMPALT_CURRENT_BG ]]; then
    print -n "%{%k%F{$__SIMPALT_CURRENT_BG}%}"
  else
    print -n "%{%k%}"
  fi
  print -n "%{%f%}"
}

### Prompt components
# Each component will draw itself, and hide itself if no information needs to be shown

# Context: user@hostname (who am I and where am I)
prompt_context() {
  local context user=`whoami`

  if [[ "$user" != "$DEFAULT_USER" || -n "$SSH_CONNECTION" ]]; then
    [[ -n "$COMPUTER_SYMBOL" ]] && context="$COMPUTER_SYMBOL" || context="$user@%m"
    prompt_segment $PRIMARY_FG default "%(!.%{%F{yellow}%}.)$context"
  fi
}

# Git: branch/detached head, dirty status
prompt_git() {
  local ref color
  is_dirty() {
    test -n "$(git status --porcelain --ignore-submodules)"
  }
  is_wip() {
    test -n "$(git log -n1 --format='%s' 2> /dev/null | grep -iw '^wip')"
  }
  ref="$vcs_info_msg_0_"

  if [ $SIMPALT_SMALL ]; then
    if [ -z "$ref" ]; then
      color=blue
    else
      if ! $(git symbolic-ref HEAD &> /dev/null); then
        color=red
      elif is_wip; then
        color=magenta
      elif is_dirty; then
        color=yellow
      else
        color=green
      fi

      if [[ "$color" == "red" ]] || [[ $SIMPALT_MAIN_BRANCHES =~ (^|[[:space:]])$ref($|[[:space:]]) ]]; then
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
        ref="$ref ±"
      else
        color=green
        ref="$ref"
      fi

      if is_wip; then
        color=magenta
      fi

      if ! $(git symbolic-ref HEAD &> /dev/null); then
        ref="➦ ${ref/.../}"
      else
        ref=" $ref"
      fi

      prompt_segment $color $PRIMARY_FG "$ref"
    fi
  fi
}

# AWS: current aws-vault session
prompt_aws() {
  if [ $AWS_VAULT ]; then
    [ $SIMPALT_SMALL ] && prompt_segment black magenta "" || prompt_segment magenta $PRIMARY_FG " $AWS_VAULT"
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
  [[ $RETVAL -ne 0 ]] && symbols+="%{%F{red}%}✘"
  [[ $UID -eq 0 ]] && symbols+="%{%F{yellow}%}☢"
  [[ $(jobs -l | wc -l) -gt 0 ]] && symbols+="%{%F{blue}%}⚙"

  [[ -n "$symbols" ]] && prompt_segment $PRIMARY_FG default "$symbols"
}

# Display current virtual environment
prompt_virtualenv() {
  if [[ -n $VIRTUAL_ENV ]]; then
    prompt_segment cyan $PRIMARY_FG "$(basename $VIRTUAL_ENV)"
  fi
}

## Main prompt
prompt_simpalt_main() {
  RETVAL=$?
  __SIMPALT_CURRENT_BG='NONE'
  unset __SIMPALT_PENDING_FLAG

  for prompt_segment in "${SIMPALT_PROMPT_SEGMENTS[@]}"; do
    [[ -n $prompt_segment ]] && $prompt_segment
  done
  prompt_end

  unset __SIMPALT_CURRENT_BG
  unset __SIMPALT_PENDING_FLAG
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
