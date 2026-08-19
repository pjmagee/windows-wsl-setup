#!/usr/bin/env bash
# Idempotent Ubuntu 26.04 WSL workstation bootstrap.
# Installs native Linux tools only. Does not use Windows interop copies.
# Safe to re-run. Does not install Linux VS Code, Discord, or Oh My Posh.
# apt = system packages. Homebrew = CLIs and language runtimes.
# Compass (Linux GUI) and Cloudflare cf stay as special steps.
# Optional toolchain steps continue after a blocked host or installer error.
# Profiles: ./install.sh <name>  (shipped: home|work — ID lists in profiles/linux/).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${HOME:=$(getent passwd "$(id -un)" | cut -d: -f6)}"
export HOME
BREW_PREFIX="/home/linuxbrew/.linuxbrew"

# Wrappers first, then cargo/GOPATH, then whatever brew shellenv prepends.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/go/bin:$PATH"
if [ -x "$BREW_PREFIX/bin/brew" ]; then
  eval "$("$BREW_PREFIX/bin/brew" shellenv bash)"
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/go/bin:$PATH"
fi

log() { printf '\n==> %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }
# True only for a native Linux binary. Windows copies on the WSL interop
# PATH (/mnt/c/..., *.exe) must not count — work laptops will not have them.
is_linux_bin() {
  have "$1" || return 1
  local p
  p="$(command -v "$1")"
  case "$p" in
    /mnt/[a-z]/*|/mnt/[A-Z]/*|*.exe) return 1 ;;
    *) return 0 ;;
  esac
}

need_sudo() {
  if ! sudo -n true 2>/dev/null; then
    echo "sudo is required and must be passwordless (sudo -n)." >&2
    echo "From Windows:  windows-wsl-setup.exe  (New WSL), or  powershell -NoProfile -ExecutionPolicy Bypass -File windows\\bootstrap.ps1" >&2
    echo "Or as root:    wsl -d Ubuntu-26.04 -u root -- bash windows/ensure-user.sh \"\$(id -un)\"" >&2
    exit 1
  fi
}

# Work laptops often block GitHub, Google, Stripe, etc. Fail fast instead of
# hanging. Not exported — child installers still use system curl.
curl() {
  command curl \
    --connect-timeout "${CURL_CONNECT_TIMEOUT:-15}" \
    --max-time "${CURL_MAX_TIME:-180}" \
    --retry "${CURL_RETRY:-2}" \
    --retry-delay 1 \
    --retry-connrefused \
    "$@"
}

# Extra apt sources from a previous run can 404/timeout on a locked-down
# network. Fetch what is reachable and keep going.
apt_update() {
  need_sudo
  if ! sudo apt-get \
    -o Acquire::Retries=2 \
    -o Acquire::http::Timeout=20 \
    -o Acquire::https::Timeout=20 \
    update -y; then
    log "apt-get update reported errors (a repo may be blocked); continuing"
  fi
}

FAILED_STEPS=()

# Run an optional installer. Required host steps are called directly and
# still abort the script under set -e.
run_step() {
  local fn="$1"
  if "$fn"; then
    return 0
  fi
  echo "!! $fn failed (unreachable host or installer error); continuing" >&2
  FAILED_STEPS+=("$fn")
  return 0
}

# Profile name → profiles/linux/<name>.json (or ~/.config/wsl-setup/profiles/).
# Overlay ~/.config/wsl-setup/linux-profile.json { "tools": ["id", ...] } replaces the ID list.
PROFILE=""
PROFILE_FILE=""
PROFILE_STEPS=()
PROFILE_STATE="$HOME/.config/wsl-setup/profile"
TOOLS_OVERLAY="$HOME/.config/wsl-setup/linux-profile.json"
LEGACY_OVERLAY="$HOME/.config/wsl-setup/tools.json"

usage() {
  echo "usage: ./install.sh [profile]"
  echo "  home   shipped home toolchain (default if nothing saved)"
  echo "  work   shipped work toolchain; drops home-only extras"
  echo "  <name> profiles/linux/<name>.json or ~/.config/wsl-setup/profiles/<name>.json"
}

read_step_file() {
  local f="$1"
  if [ ! -f "$f" ]; then
    echo "missing profile file: $f" >&2
    return 1
  fi
  tr -d '\r' <"$f" | grep -vE '^\s*(#|$)' || true
}

find_profile_file() {
  local n="$1"
  if [ -f "$HOME/.config/wsl-setup/profiles/${n}.json" ]; then
    printf '%s\n' "$HOME/.config/wsl-setup/profiles/${n}.json"
    return 0
  fi
  if [ -f "$ROOT/profiles/linux/${n}.json" ]; then
    printf '%s\n' "$ROOT/profiles/linux/${n}.json"
    return 0
  fi
  return 1
}

resolve_profile() {
  local requested="${1:-${WSL_SETUP_PROFILE:-}}"
  if [ -z "$requested" ] && [ -f "$PROFILE_STATE" ]; then
    requested="$(tr -d '[:space:]' <"$PROFILE_STATE")"
  fi
  # Old checkouts saved "universal". That profile is gone; treat as home.
  if [ "$requested" = "universal" ]; then
    requested="home"
  fi
  requested="${requested:-home}"
  case "$requested" in
    -h|--help|help) usage; exit 0 ;;
  esac
  if ! PROFILE_FILE="$(find_profile_file "$requested")"; then
    echo "unknown profile: $requested (no profiles/linux/${requested}.json)" >&2
    usage >&2
    exit 1
  fi
  PROFILE="$requested"
  mkdir -p "$(dirname "$PROFILE_STATE")"
  printf '%s\n' "$PROFILE" >"$PROFILE_STATE"
}

collect_steps() {
  local line extra
  PROFILE_STEPS=()
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    PROFILE_STEPS+=("$line")
  done < <(read_step_file "$ROOT/profiles/base.txt")
  extra="$ROOT/profiles/${PROFILE}.txt"
  if [ -f "$extra" ]; then
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      PROFILE_STEPS+=("$line")
    done < <(read_step_file "$extra")
  fi
}

ensure_brew_env() {
  if [ -x "$BREW_PREFIX/bin/brew" ]; then
    eval "$("$BREW_PREFIX/bin/brew" shellenv bash)"
  elif is_linux_bin brew; then
    eval "$(brew shellenv bash)"
  else
    return 1
  fi
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/go/bin:$PATH"
  if [ -n "${HOMEBREW_PREFIX:-}" ] && [ -d "$HOMEBREW_PREFIX/opt/dotnet/libexec" ]; then
    export DOTNET_ROOT="$HOMEBREW_PREFIX/opt/dotnet/libexec"
  fi
  if [ -n "${HOMEBREW_PREFIX:-}" ] && [ -d "$HOMEBREW_PREFIX/share/google-cloud-sdk/bin" ]; then
    export PATH="$PATH:$HOMEBREW_PREFIX/share/google-cloud-sdk/bin"
  fi
  export HOMEBREW_NO_AUTO_UPDATE="${HOMEBREW_NO_AUTO_UPDATE:-1}"
  export HOMEBREW_NO_ENV_HINTS=1
  export NONINTERACTIVE=1
  return 0
}

# Uninstall catalog packages not on this profile.
prune_unselected_extras() {
  log "pruning extras not on profile=$PROFILE"
  local line kind pkg overlay=""
  if [ -f "$TOOLS_OVERLAY" ]; then
    overlay="$TOOLS_OVERLAY"
  elif [ -f "$LEGACY_OVERLAY" ]; then
    overlay="$LEGACY_OVERLAY"
  fi
  local args=("$ROOT/scripts/linux-tools.py" prune "$PROFILE" "$ROOT/profiles/linux.json" "$PROFILE_FILE")
  if [ -n "$overlay" ]; then
    args+=("$overlay")
  fi
  while IFS=$'\t' read -r kind pkg; do
    [ -n "$pkg" ] || continue
    case "$pkg" in
      opencode)
        rm -f "$HOME/.local/bin/opencode" "$HOME/.opencode/bin/opencode"
        ;;
      claude-code)
        rm -f "$HOME/.local/bin/claude" "$HOME/.claude/bin/claude"
        ;;
      grok-build)
        rm -f "$HOME/.local/bin/grok" "$HOME/.grok/bin/grok"
        ;;
      devtunnel)
        rm -f "$HOME/.local/bin/devtunnel" "$HOME/bin/devtunnel"
        ;;
      changie) rm -f "$HOME/.local/bin/changie" ;;
      hugo) rm -f "$HOME/.local/bin/hugo" ;;
      copilot-cli) rm -f "$HOME/.local/bin/copilot" ;;
      stripe-cli)
        if dpkg-query -W -f='${Status}' stripe 2>/dev/null | grep -q 'install ok installed'; then
          need_sudo
          sudo DEBIAN_FRONTEND=noninteractive apt-get remove -y stripe || true
        fi
        if [ -f /etc/apt/sources.list.d/stripe.list ]; then
          need_sudo
          sudo rm -f /etc/apt/sources.list.d/stripe.list
        fi
        ;;
    esac
    if ensure_brew_env; then
      if [ "$kind" = cask ]; then
        brew uninstall --cask --force "$pkg" >/dev/null 2>&1 || true
      else
        brew uninstall --force "$pkg" >/dev/null 2>&1 || true
      fi
    fi
  done < <(python3 "${args[@]}")
}

# Drop [user] / [interop] and rewrite them. Other sections stay.
write_wsl_conf() {
  local user_name="$1"
  local conf=/etc/wsl.conf
  local tmp
  tmp="$(mktemp)"
  if [ -f "$conf" ]; then
    awk '
      /^\[user\]/ {skip=1; next}
      /^\[interop\]/ {skip=1; next}
      /^\[/ {skip=0}
      !skip {print}
    ' "$conf" >"$tmp"
  else
    : >"$tmp"
  fi
  {
    awk '{ lines[NR]=$0 } END { n=NR; while (n>0 && lines[n]=="") n--; for (i=1;i<=n;i++) print lines[i] }' "$tmp"
    printf '\n[user]\ndefault=%s\n\n[interop]\nenabled=true\n' "$user_name"
  } | sudo tee "$conf" >/dev/null
  rm -f "$tmp"
}

ensure_passwordless_sudo() {
  log "passwordless sudo + /etc/wsl.conf"
  need_sudo
  local u
  u="$(id -un)"
  printf '%s ALL=(ALL) NOPASSWD:ALL\n' "$u" | sudo tee "/etc/sudoers.d/90-${u}" >/dev/null
  sudo chmod 440 "/etc/sudoers.d/90-${u}"
  sudo visudo -cf "/etc/sudoers.d/90-${u}" >/dev/null
  write_wsl_conf "$u"
}

install_apt() {
  need_sudo
  log "apt packages"
  mapfile -t pkgs < <(tr -d '\r' <"$ROOT/packages/apt.txt" | grep -vE '^\s*(#|$)')
  apt_update
  if ! sudo DEBIAN_FRONTEND=noninteractive apt-get install -y "${pkgs[@]}"; then
    log "batch apt install failed; trying packages one by one"
    local pkg
    for pkg in "${pkgs[@]}"; do
      if ! sudo DEBIAN_FRONTEND=noninteractive apt-get install -y "$pkg"; then
        echo "!! apt package $pkg failed; continuing" >&2
        FAILED_STEPS+=("apt:$pkg")
      fi
    done
  fi
  if have fdfind && ! have fd; then
    mkdir -p "$HOME/.local/bin"
    ln -sfn "$(command -v fdfind)" "$HOME/.local/bin/fd"
  fi
}

ensure_bashrc() {
  local bashrc="$HOME/.bashrc"
  [ -f "$bashrc" ] || return 0

  if grep -q '>>> oh-my-posh >>>' "$bashrc" || grep -q 'oh-my-posh init' "$bashrc"; then
    log "removing Oh My Posh from ~/.bashrc (Windows-only)"
    local tmp
    tmp="$(mktemp)"
    awk '
      />>> oh-my-posh >>>/ {skip=1; next}
      /<<< oh-my-posh <<</ {skip=0; next}
      /oh-my-posh init/ {next}
      !skip {print}
    ' "$bashrc" >"$tmp"
    mv "$tmp" "$bashrc"
  fi

  local path_block tmp stripped
  path_block="$(mktemp)"
  tmp="$(mktemp)"
  stripped="$(mktemp)"
  cat >"$path_block" <<'EOF'
# >>> wsl-linux-path >>>
# Linux toolchains must precede the Windows PATH that WSL appends.
# Homebrew owns CLIs. ~/.local/bin is wrappers (wsl-open, compass, python3.14).
if [ -x /home/linuxbrew/.linuxbrew/bin/brew ]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/go/bin:$PATH"
if [ -n "${HOMEBREW_PREFIX:-}" ] && [ -d "$HOMEBREW_PREFIX/share/google-cloud-sdk/bin" ]; then
  export PATH="$PATH:$HOMEBREW_PREFIX/share/google-cloud-sdk/bin"
fi
if [ -n "${HOMEBREW_PREFIX:-}" ] && [ -d "$HOMEBREW_PREFIX/opt/dotnet/libexec" ]; then
  export DOTNET_ROOT="$HOMEBREW_PREFIX/opt/dotnet/libexec"
fi
if command -v fnm >/dev/null 2>&1; then
  eval "$(fnm env --shell bash 2>/dev/null)" || true
fi
# Linux GUI/CLI "open this URL" should hit the Windows default browser.
# Ubuntu 26.04 dropped wslu/wslview (discontinued upstream).
export BROWSER="$HOME/.local/bin/wsl-open"
export GH_BROWSER="$HOME/.local/bin/wsl-open"
# <<< wsl-linux-path <<<

EOF
  awk '
    />>> wsl-linux-path >>>/ {skip=1; next}
    /<<< wsl-linux-path <<</ {skip=0; next}
    /^# Linux toolchains must precede the Windows PATH/ {next}
    !skip {print}
  ' "$bashrc" >"$stripped"
  log "refreshing Linux-first PATH in ~/.bashrc"
  cat "$path_block" "$stripped" >"$tmp"
  mv "$tmp" "$bashrc"
  rm -f "$path_block" "$stripped"

  if ! grep -q '>>> wsl-shell >>>' "$bashrc"; then
    log "adding starship / zoxide / atuin / fzf to ~/.bashrc"
    cat >>"$bashrc" <<'EOF'

# >>> wsl-shell >>>
command -v starship >/dev/null && eval "$(starship init bash)"
command -v zoxide >/dev/null && eval "$(zoxide init bash)"
command -v atuin >/dev/null && eval "$(atuin init bash)"
command -v fzf >/dev/null && eval "$(fzf --bash 2>/dev/null)" || true
# <<< wsl-shell <<<
EOF
  fi

  if ! grep -q 'bash_aliases' "$bashrc"; then
    cat >>"$bashrc" <<'EOF'

if [ -f ~/.bash_aliases ]; then
  . ~/.bash_aliases
fi
EOF
  fi
}

# Replace or append a marked block. Safe to re-run.
upsert_marked_block() {
  local file="$1"
  local begin="$2"
  local end="$3"
  local block="$4"
  local tmp stripped
  mkdir -p "$(dirname "$file")"
  [ -f "$file" ] || : >"$file"
  tmp="$(mktemp)"
  stripped="$(mktemp)"
  awk -v b="$begin" -v e="$end" '
    index($0, b) {skip=1; next}
    index($0, e) {skip=0; next}
    !skip {print}
  ' "$file" >"$stripped"
  awk '{ lines[NR]=$0 } END { n=NR; while (n>0 && lines[n]=="") n--; for (i=1;i<=n;i++) print lines[i] }' \
    "$stripped" >"$tmp"
  mv "$tmp" "$stripped"
  tmp="$(mktemp)"
  {
    cat "$stripped"
    [ -s "$stripped" ] && printf '\n'
    printf '%s\n' "$block"
  } >"$tmp"
  mv "$tmp" "$file"
  rm -f "$stripped"
}

# Drop an older unmarked copy of the two aliases. No-op once the
# marked block is present.
strip_unmarked_1password_ssh() {
  local file="$1"
  local begin="$2"
  [ -f "$file" ] || return 0
  grep -q "$begin" "$file" && return 0
  local tmp
  tmp="$(mktemp)"
  grep -vE "^alias ssh='ssh\\.exe'$|^alias ssh-add='ssh-add\\.exe'$|^# 1Password SSH agent|^# https://www.1password.dev/ssh/integrations/wsl$" \
    "$file" >"$tmp" || true
  mv "$tmp" "$file"
}

ensure_1password_ssh() {
  log "1Password SSH agent (ssh.exe aliases)"
  local begin='>>> wsl-1password-ssh >>>'
  local end='<<< wsl-1password-ssh <<<'
  local block
  block="$(cat <<'EOF'
# >>> wsl-1password-ssh >>>
# 1Password SSH agent (WSL -> Windows). https://www.1password.dev/ssh/integrations/wsl
alias ssh='ssh.exe'
alias ssh-add='ssh-add.exe'
# <<< wsl-1password-ssh <<<
EOF
)"
  strip_unmarked_1password_ssh "$HOME/.bash_aliases" "$begin"
  strip_unmarked_1password_ssh "$HOME/.zshrc" "$begin"
  upsert_marked_block "$HOME/.bash_aliases" "$begin" "$end" "$block"
  upsert_marked_block "$HOME/.zshrc" "$begin" "$end" "$block"

  if have git; then
    git config --global core.sshCommand ssh.exe
  fi
}

remove_oh_my_posh() {
  log "ensuring Oh My Posh is not installed in WSL"
  rm -f "$HOME/.local/bin/oh-my-posh"
}

install_wsl_open() {
  log "wsl-open (Windows default browser)"
  mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
  install -m 0755 "$ROOT/scripts/wsl-open" "$HOME/.local/bin/wsl-open"
  rm -f "$HOME/.local/bin/xdg-open"
  cat >"$HOME/.local/share/applications/wsl-open.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Windows Browser
Comment=Open links in the Windows default browser
Exec=$HOME/.local/bin/wsl-open %u
Terminal=false
NoDisplay=true
MimeType=x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/mailto;x-scheme-handler/ftp;
EOF
  if have xdg-mime; then
    xdg-mime default wsl-open.desktop x-scheme-handler/http || true
    xdg-mime default wsl-open.desktop x-scheme-handler/https || true
    xdg-mime default wsl-open.desktop x-scheme-handler/mailto || true
  fi
}

install_homebrew() {
  log "Homebrew (Linux / WSL)"
  if ensure_brew_env; then
    return 0
  fi
  need_sudo
  # Official prefix /home/linuxbrew/.linuxbrew is required for bottles on Linux.
  # Child bash — our curl() wrapper is not exported.
  NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  ensure_brew_env
}

# formula<TAB>name  or  cask<TAB>name
read_brewfile_entries() {
  local f="$1"
  [ -f "$f" ] || return 0
  tr -d '\r' <"$f" | sed -n \
    -e 's/^[[:space:]]*brew[[:space:]]*"\([^"]*\)".*/formula\t\1/p' \
    -e 's/^[[:space:]]*cask[[:space:]]*"\([^"]*\)".*/cask\t\1/p'
}

install_one_brew_entry() {
  local kind="$1"
  local name="$2"
  case "$kind" in
    formula)
      if brew list --formula "$name" >/dev/null 2>&1; then
        brew upgrade --formula "$name"
      else
        brew install --formula "$name"
      fi
      ;;
    cask)
      if brew list --cask "$name" >/dev/null 2>&1; then
        brew upgrade --cask "$name"
      else
        brew install --cask "$name"
      fi
      ;;
    *)
      echo "unknown brew entry kind: $kind" >&2
      return 1
      ;;
  esac
}

bundle_brewfile() {
  local file="$1"
  local kind name
  [ -f "$file" ] || return 0
  log "brew bundle $(basename "$file")"
  if brew bundle --help 2>/dev/null | grep -q -- '--upgrade'; then
    if brew bundle --file="$file" --upgrade; then
      return 0
    fi
  elif brew bundle --file="$file"; then
    brew upgrade || true
    return 0
  fi
  log "brew bundle failed for $(basename "$file"); installing one by one"
  while IFS=$'\t' read -r kind name; do
    [ -n "$name" ] || continue
    if ! install_one_brew_entry "$kind" "$name"; then
      echo "!! brew $kind $name failed; continuing" >&2
      FAILED_STEPS+=("brew:$kind:$name")
    fi
  done < <(read_brewfile_entries "$file")
}

# Drop leftover tarball / vendor-apt copies so they cannot shadow brew.
# Only removes a path after brew actually provides that command.
migrate_legacy_clis() {
  local prefix="${HOMEBREW_PREFIX:-$BREW_PREFIX}"
  brew_bin() { [ -x "$prefix/bin/$1" ]; }

  if brew_bin gh; then rm -f "$HOME/.local/bin/gh"; fi
  if brew_bin dagger; then rm -f "$HOME/.local/bin/dagger"; fi
  if brew_bin starship; then rm -f "$HOME/.local/bin/starship"; fi
  if brew_bin zoxide; then rm -f "$HOME/.local/bin/zoxide"; fi
  if brew_bin atuin; then rm -f "$HOME/.local/bin/atuin" "$HOME/.atuin/bin/atuin"; fi
  if brew_bin bun; then rm -f "$HOME/.bun/bin/bun"; fi
  if brew_bin uv; then rm -f "$HOME/.local/bin/uv"; fi
  if brew_bin fnm; then rm -f "$HOME/.local/share/fnm/fnm"; fi
  if brew_bin go; then rm -rf "$HOME/.local/go"; fi
  if brew_bin saml2aws; then rm -f "$HOME/.local/bin/saml2aws"; fi
  if brew_bin helm; then rm -f "$HOME/.local/bin/helm"; fi
  if brew_bin 7zz; then
    rm -f "$HOME/.local/bin/7zz"
    # Keep a 7z name if brew did not provide one.
    if [ ! -x "$prefix/bin/7z" ]; then
      ln -sfn "$prefix/bin/7zz" "$HOME/.local/bin/7z"
    else
      rm -f "$HOME/.local/bin/7z"
    fi
  fi
  if brew_bin mongosh; then
    rm -f "$HOME/.local/bin/mongosh"
    rm -rf "$HOME/.local/opt/mongosh"
  fi
  if brew_bin flux; then rm -f "$HOME/.local/bin/flux"; fi
  if brew_bin opencode; then rm -f "$HOME/.local/bin/opencode" "$HOME/.opencode/bin/opencode"; fi
  if brew_bin changie; then rm -f "$HOME/.local/bin/changie"; fi
  if brew_bin hugo; then rm -f "$HOME/.local/bin/hugo"; fi
  if brew_bin copilot; then rm -f "$HOME/.local/bin/copilot"; fi
  if brew_bin claude; then rm -f "$HOME/.local/bin/claude" "$HOME/.claude/bin/claude"; fi
  if brew_bin grok; then rm -f "$HOME/.local/bin/grok" "$HOME/.grok/bin/grok"; fi
  if brew_bin devtunnel; then rm -f "$HOME/.local/bin/devtunnel" "$HOME/bin/devtunnel"; fi
  if brew_bin azd; then rm -f "$HOME/.local/bin/azd"; fi

  if brew_bin wrangler && is_linux_bin npm; then
    npm uninstall -g wrangler >/dev/null 2>&1 || true
  fi

  if brew_bin aws && [ -x /usr/local/bin/aws ]; then
    need_sudo
    sudo rm -f /usr/local/bin/aws /usr/local/bin/aws_completer
    sudo rm -rf /usr/local/aws-cli
  fi
  if brew_bin pwsh && [ -L /usr/local/bin/pwsh ]; then
    need_sudo
    sudo rm -f /usr/local/bin/pwsh
  fi

  if ! sudo -n true 2>/dev/null; then
    return 0
  fi
  local pkg
  for pkg in 1password-cli azure-cli google-cloud-cli cloudflared stripe powershell; do
    case "$pkg" in
      1password-cli) brew_bin op || continue ;;
      azure-cli) brew_bin az || continue ;;
      google-cloud-cli) brew_bin gcloud || continue ;;
      cloudflared) brew_bin cloudflared || continue ;;
      stripe) brew_bin stripe || continue ;;
      powershell) brew_bin pwsh || continue ;;
    esac
    if dpkg-query -W -f='${Status}' "$pkg" 2>/dev/null | grep -q 'install ok installed'; then
      sudo DEBIAN_FRONTEND=noninteractive apt-get remove -y "$pkg" || true
    fi
  done
  sudo rm -f \
    /etc/apt/sources.list.d/1password.list \
    /etc/apt/sources.list.d/azure-cli.sources \
    /etc/apt/sources.list.d/google-cloud-sdk.list \
    /etc/apt/sources.list.d/cloudflared.list \
    /etc/apt/sources.list.d/stripe.list
}

install_brew() {
  log "Homebrew packages (profile=$PROFILE)"
  if ! ensure_brew_env; then
    echo "Homebrew is not installed (install_homebrew must succeed first)" >&2
    return 1
  fi
  # One index refresh per run. Individual brew install uses NO_AUTO_UPDATE.
  HOMEBREW_NO_AUTO_UPDATE=0 brew update || log "brew update reported errors (GitHub may be blocked); continuing"
  local generated
  generated="$(mktemp)"
  local overlay=""
  if [ -f "$TOOLS_OVERLAY" ]; then
    overlay="$TOOLS_OVERLAY"
  elif [ -f "$LEGACY_OVERLAY" ]; then
    overlay="$LEGACY_OVERLAY"
  fi
  if [ -n "$overlay" ]; then
    python3 "$ROOT/scripts/linux-tools.py" brewfile "$PROFILE" "$ROOT/profiles/linux.json" "$PROFILE_FILE" "$overlay" >"$generated"
  else
    python3 "$ROOT/scripts/linux-tools.py" brewfile "$PROFILE" "$ROOT/profiles/linux.json" "$PROFILE_FILE" >"$generated"
  fi
  bundle_brewfile "$generated"
  rm -f "$generated"
  migrate_legacy_clis
}

install_uv_python() {
  log "Python 3.14 (uv)"
  if ! is_linux_bin uv; then
    echo "uv missing (brew package uv)" >&2
    return 1
  fi
  uv python install 3.14
  local py
  py="$(uv python find 3.14)"
  mkdir -p "$HOME/.local/bin"
  ln -sfn "$py" "$HOME/.local/bin/python3.14"
}

install_rust() {
  log "rustup toolchain"
  if ! is_linux_bin rustup; then
    echo "rustup missing (brew package rustup)" >&2
    return 1
  fi
  rustup default stable
  rustup update stable
}

install_fnm_node() {
  log "Node LTS (fnm)"
  if ! is_linux_bin fnm; then
    echo "fnm missing (brew package fnm)" >&2
    return 1
  fi
  eval "$(fnm env --shell bash)"
  fnm install --lts
  fnm default lts-latest
}

install_starship_config() {
  log "starship config"
  mkdir -p "$HOME/.config"
  if [ ! -f "$HOME/.config/starship.toml" ]; then
    install -m 0644 "$ROOT/starship.toml" "$HOME/.config/starship.toml"
  fi
}

# Cloudflare's unified `cf` CLI is not in Homebrew (npm only, preview).
install_cloudflare_cf() {
  log "Cloudflare CLI (cf)"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  if ! is_linux_bin npm; then
    echo "Linux npm is required for cf (install_fnm_node first)" >&2
    return 1
  fi
  npm install -g cf@latest
}

# Electron's Chromium sandbox fails under WSL. Compass also rejects unknown
# Chromium flags unless --ignore-additional-command-line-flags is set.
# CHROME_DESKTOP must not contain spaces: Electron asks systemd for
# app-${CHROME_DESKTOP}-${pid}.scope and "MongoDB Compass" is an invalid unit name.
ensure_compass_wsl_wrapper() {
  mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
  cat >"$HOME/.local/bin/mongodb-compass" <<'EOF'
#!/usr/bin/env bash
# Linux Compass via WSLg. COMPASS_WINDOWS=1 starts the Windows EXE instead.
win_compass() {
  local c f
  for c in \
    "/mnt/c/Program Files/MongoDB Compass/MongoDBCompass.exe" \
    "/mnt/c/Program Files (x86)/MongoDB Compass/MongoDBCompass.exe"
  do
    [ -e "$c" ] && { printf '%s\n' "$c"; return 0; }
  done
  f="$(ls -1 /mnt/c/Users/*/AppData/Local/MongoDBCompass/MongoDBCompass.exe \
           /mnt/c/Users/*/AppData/Local/Programs/MongoDBCompass/MongoDBCompass.exe \
           2>/dev/null | head -n1)"
  [ -n "$f" ] && printf '%s\n' "$f"
}

if [ "${COMPASS_WINDOWS:-}" = 1 ]; then
  exe="$(win_compass || true)"
  if [ -z "$exe" ]; then
    echo "compass: COMPASS_WINDOWS=1 but MongoDBCompass.exe was not found." >&2
    exit 1
  fi
  exec "$exe" "$@"
fi

if [ ! -x /usr/bin/mongodb-compass ]; then
  echo "compass: Linux mongodb-compass is not installed." >&2
  exit 1
fi

# Electron 41 does not map a window on Mesa D3D12 (WSL NVIDIA GL).
export CHROME_DESKTOP=mongodb-compass.desktop
export ELECTRON_OZONE_PLATFORM_HINT=x11
export GDK_BACKEND=x11
export XDG_SESSION_TYPE=x11
export BROWSER="${BROWSER:-$HOME/.local/bin/wsl-open}"
export NODE_NO_WARNINGS=1
unset WAYLAND_DISPLAY
unset GALLIUM_DRIVER
unset XDG_CURRENT_DESKTOP
export DISPLAY="${DISPLAY:-:0}"
exec /usr/bin/mongodb-compass \
  --no-sandbox \
  --no-installURLHandlers \
  --ignore-additional-command-line-flags \
  --ozone-platform=x11 \
  --disable-gpu \
  --enable-unsafe-swiftshader \
  "$@" 2> >(grep -v --line-buffered -E 'StartTransientUnit|unknown desktop environment|default-url-scheme-handler|DEP0040|punycode|trace-deprecation' >&2)
EOF
  chmod +x "$HOME/.local/bin/mongodb-compass"
  ln -sfn "$HOME/.local/bin/mongodb-compass" "$HOME/.local/bin/compass"
  if [ -w /usr/local/bin ] || sudo -n true 2>/dev/null; then
    sudo ln -sfn "$HOME/.local/bin/mongodb-compass" /usr/local/bin/mongodb-compass
    sudo ln -sfn "$HOME/.local/bin/mongodb-compass" /usr/local/bin/compass
  fi
  cat >"$HOME/.local/share/applications/mongodb-compass.desktop" <<EOF
[Desktop Entry]
Name=MongoDB Compass
Comment=The MongoDB GUI
Exec=$HOME/.local/bin/mongodb-compass %U
Terminal=false
Type=Application
Icon=mongodb-compass
Categories=Development;Database;
StartupWMClass=MongoDB Compass
EOF
}

install_compass() {
  log "MongoDB Compass (Linux GUI via WSLg)"
  need_sudo
  local ver installed tmp deb
  ver="$(curl -fsSL https://api.github.com/repos/mongodb-js/compass/releases/latest | sed -n 's/.*"tag_name": "v\([^"]*\)".*/\1/p' | head -n1)"
  if [ -z "$ver" ]; then
    echo "could not resolve latest MongoDB Compass release" >&2
    return 1
  fi
  installed="$(dpkg-query -W -f='${Version}' mongodb-compass 2>/dev/null || true)"
  case "$installed" in
    "$ver"|"$ver"-*)
      ensure_compass_wsl_wrapper
      return 0
      ;;
  esac
  tmp="$(mktemp -d)"
  deb="$tmp/mongodb-compass_${ver}_amd64.deb"
  if ! curl -fsSL "https://downloads.mongodb.com/compass/mongodb-compass_${ver}_amd64.deb" -o "$deb"; then
    curl -fsSL "https://github.com/mongodb-js/compass/releases/download/v${ver}/mongodb-compass_${ver}_amd64.deb" -o "$deb"
  fi
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y "$deb"
  rm -rf "$tmp"
  ensure_compass_wsl_wrapper
}

# First line of stdout, even if the tool exits non-zero (stripe phones home).
fmt_ver() {
  local out
  out="$("$@" 2>/dev/null | head -n1 || true)"
  printf '%s' "${out:-missing}"
}

print_summary() {
  log "versions"
  printf 'profile    %s\n' "$PROFILE"
  printf 'brew       %s\n' "$(fmt_ver brew --version)"
  printf 'git        %s\n' "$(fmt_ver git --version)"
  printf 'gh         %s\n' "$(fmt_ver gh --version)"
  printf 'pwsh       %s\n' "$(fmt_ver pwsh --version)"
  if is_linux_bin docker; then
    printf 'docker     %s\n' "$(fmt_ver docker version --format '{{.Server.Version}}')"
  else
    printf 'docker     %s\n' "missing — Docker Desktop on Windows + enable this distro"
  fi
  printf 'node       %s\n' "$(fmt_ver node --version)"
  printf 'bun        %s\n' "$(fmt_ver bun --version)"
  printf 'go         %s\n' "$(fmt_ver go version)"
  printf 'dotnet     %s\n' "$(fmt_ver dotnet --version)"
  printf 'python     %s\n' "$(fmt_ver python3 --version)"
  printf 'python3.14 %s\n' "$(fmt_ver python3.14 --version)"
  printf 'uv         %s\n' "$(fmt_ver uv --version)"
  printf 'rustc      %s\n' "$(fmt_ver rustc --version)"
  printf 'op         %s\n' "$(fmt_ver op --version)"
  printf 'dagger     %s\n' "$(fmt_ver dagger version)"
  printf 'starship   %s\n' "$(fmt_ver starship --version)"
  printf 'zoxide     %s\n' "$(fmt_ver zoxide --version)"
  printf 'fzf        %s\n' "$(fmt_ver fzf --version)"
  printf 'atuin      %s\n' "$(fmt_ver atuin --version)"
  printf 'opencode   %s\n' "$(fmt_ver opencode --version)"
  printf 'copilot    %s\n' "$(fmt_ver copilot --version)"
  printf 'claude     %s\n' "$(fmt_ver claude --version)"
  printf 'grok       %s\n' "$(fmt_ver grok --version)"
  printf 'az         %s\n' "$(az version -o json 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["azure-cli"])' 2>/dev/null || echo missing)"
  printf 'azd        %s\n' "$(fmt_ver azd version)"
  printf 'gcloud     %s\n' "$(fmt_ver gcloud --version)"
  printf 'saml2aws   %s\n' "$(saml2aws --version 2>&1 | head -n1 || echo missing)"
  printf 'aws        %s\n' "$(fmt_ver aws --version)"
  printf 'cf         %s\n' "$(fmt_ver cf --version)"
  printf 'wrangler   %s\n' "$(fmt_ver wrangler --version)"
  printf 'cloudflared %s\n' "$(fmt_ver cloudflared --version)"
  printf 'compass    %s\n' "$(dpkg-query -W -f='${Version}' mongodb-compass 2>/dev/null || echo missing)"
  printf 'devtunnel  %s\n' "$(fmt_ver devtunnel --version)"
  printf 'changie    %s\n' "$(fmt_ver changie --version)"
  printf 'helm       %s\n' "$(fmt_ver helm version --short)"
  printf 'hugo       %s\n' "$(fmt_ver hugo version)"
  printf 'stripe     %s\n' "$(fmt_ver stripe version)"
  if is_linux_bin 7zz; then
    printf '7zz        %s\n' "$(7zz 2>&1 | head -n1 || true)"
  else
    printf '7zz        missing\n'
  fi
  printf 'mongosh    %s\n' "$(fmt_ver mongosh --version)"
  printf 'flux       %s\n' "$(fmt_ver flux --version)"
  printf 'oh-my-posh %s\n' "$(command -v oh-my-posh >/dev/null && echo 'STILL PRESENT (should be Windows-only)' || echo 'not in WSL (ok)')"
  if grep -q "alias ssh='ssh.exe'" "$HOME/.bash_aliases" 2>/dev/null; then
    printf '1p-ssh     aliases -> ssh.exe\n'
  else
    printf '1p-ssh     aliases missing\n'
  fi
  printf 'git-ssh    %s\n' "$(git config --global --get core.sshCommand 2>/dev/null || echo unset)"
  if sudo -n true 2>/dev/null; then
    printf 'sudo       passwordless\n'
  else
    printf 'sudo       PASSWORD REQUIRED (run windows/bootstrap.ps1)\n'
  fi
  printf 'wsl.conf   default=%s\n' "$(awk -F= '/^\[user\]/{s=1;next} /^\[/{s=0} s&&$1=="default"{print $2}' /etc/wsl.conf 2>/dev/null || echo unset)"
  if ((${#FAILED_STEPS[@]} > 0)); then
    log "failed this run (re-run ./install.sh when the host is reachable)"
    local s
    for s in "${FAILED_STEPS[@]}"; do
      printf '  %s\n' "$s"
    done
  fi
}

main() {
  resolve_profile "${1:-}"
  collect_steps
  log "profile $PROFILE"
  mkdir -p "$HOME/.local/bin" "$HOME/code"
  remove_oh_my_posh
  ensure_bashrc
  ensure_passwordless_sudo
  ensure_1password_ssh
  install_wsl_open
  prune_unselected_extras
  local fn
  for fn in "${PROFILE_STEPS[@]}"; do
    if ! declare -F "$fn" >/dev/null; then
      echo "!! unknown step $fn (check profiles/${PROFILE}.txt)" >&2
      FAILED_STEPS+=("$fn")
      continue
    fi
    run_step "$fn"
  done
  ensure_brew_env || true
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  print_summary
  echo
  echo "Prompt in WSL is Starship. Oh My Posh stays on Windows only."
  echo "VS Code stays on Windows. From a Linux path:  code ."
  echo "MongoDB Compass is a Linux GUI:  compass    (window appears on Windows via WSLg)"
  echo "CLIs update with:  brew update && brew upgrade"
  case "$PROFILE" in
    work)
      echo "Work profile: profiles/linux/work.json (Copilot CLI; no grok/claude/opencode)."
      ;;
    home)
      echo "Home profile: profiles/linux/home.json (grok/claude/opencode)."
      ;;
  esac
  if ((${#FAILED_STEPS[@]} > 0)); then
    echo "Some tools were skipped (blocked host or installer error). Re-run ./install.sh later."
  fi
}

main "$@"
