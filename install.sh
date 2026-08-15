#!/usr/bin/env bash
# Idempotent Ubuntu 26.04 WSL workstation bootstrap.
# Safe to re-run. Does not install Linux VS Code, Discord, or Oh My Posh.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${HOME:=$(getent passwd "$(id -un)" | cut -d: -f6)}"
export HOME
export PATH="$HOME/.local/bin:$HOME/.atuin/bin:$HOME/.opencode/bin:$HOME/.bun/bin:$HOME/.dotnet:$HOME/.dotnet/tools:$HOME/.local/go/bin:$HOME/go/bin:$HOME/.cargo/bin:$HOME/.grok/bin:$PATH"

log() { printf '\n==> %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }
is_linux_bin() {
  have "$1" || return 1
  case "$(command -v "$1")" in
    /mnt/c/*|/mnt/d/*) return 1 ;;
    *) return 0 ;;
  esac
}

need_sudo() {
  if ! sudo -n true 2>/dev/null; then
    echo "sudo is required for apt packages. Enable NOPASSWD or run this in a TTY." >&2
    exit 1
  fi
}

install_apt() {
  need_sudo
  log "apt packages"
  mapfile -t pkgs < <(grep -vE '^\s*(#|$)' "$ROOT/packages/apt.txt")
  sudo apt-get update -y
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y "${pkgs[@]}"
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

  if ! grep -q '>>> wsl-linux-path >>>' "$bashrc"; then
    log "adding Linux-first PATH to ~/.bashrc"
    local tmp
    tmp="$(mktemp)"
    cat >"$tmp" <<'EOF'
# Linux toolchains must precede the Windows PATH that WSL appends.
# >>> wsl-linux-path >>>
export PATH="$HOME/.local/bin:$HOME/.atuin/bin:$HOME/.opencode/bin:$HOME/.bun/bin:$HOME/.dotnet:$HOME/.dotnet/tools:$HOME/.local/go/bin:$HOME/go/bin:$HOME/.cargo/bin:$HOME/.grok/bin:$PATH"
if [ -d "$HOME/.local/share/fnm" ]; then
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
fi
# <<< wsl-linux-path <<<

EOF
    cat "$bashrc" >>"$tmp"
    mv "$tmp" "$bashrc"
  fi

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
}

remove_oh_my_posh() {
  log "ensuring Oh My Posh is not installed in WSL"
  rm -f "$HOME/.local/bin/oh-my-posh"
}

install_uv_python() {
  log "uv + Python 3.14"
  if ! have uv; then
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
  fi
  uv python install 3.14
  local py
  py="$(uv python find 3.14)"
  mkdir -p "$HOME/.local/bin"
  ln -sfn "$py" "$HOME/.local/bin/python3.14"
}

install_rust() {
  log "rustup"
  if ! have rustup; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  fi
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
  rustup default stable
  rustup update stable
}

install_bun() {
  log "bun"
  if [ -x "$HOME/.bun/bin/bun" ]; then
    "$HOME/.bun/bin/bun" upgrade || true
  else
    curl -fsSL https://bun.com/install | bash
  fi
}

install_fnm_node() {
  log "fnm + Node LTS"
  if [ ! -x "$HOME/.local/share/fnm/fnm" ] && ! have fnm; then
    curl -fsSL https://fnm.vercel.app/install | bash -s -- --install-dir "$HOME/.local/share/fnm" --skip-shell
  fi
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash)"
  fnm install --lts
  fnm default lts-latest
}

install_go() {
  log "go"
  local gover
  gover="$(curl -fsSL 'https://go.dev/VERSION?m=text' | head -n1)"
  if [ ! -x "$HOME/.local/go/bin/go" ] || [ "$("$HOME/.local/go/bin/go" env GOVERSION 2>/dev/null || true)" != "$gover" ]; then
    local tmp
    tmp="$(mktemp -d)"
    curl -fsSL "https://go.dev/dl/${gover}.linux-amd64.tar.gz" -o "$tmp/go.tgz"
    rm -rf "$HOME/.local/go"
    mkdir -p "$HOME/.local"
    tar -C "$HOME/.local" -xzf "$tmp/go.tgz"
    rm -rf "$tmp"
  fi
}

install_dotnet() {
  log "dotnet 10"
  local tmp
  tmp="$(mktemp)"
  curl -fsSL https://dot.net/v1/dotnet-install.sh -o "$tmp"
  bash "$tmp" --channel 10.0 --install-dir "$HOME/.dotnet"
  rm -f "$tmp"
}

install_dagger() {
  log "dagger (Linux)"
  if is_linux_bin dagger; then
    return 0
  fi
  mkdir -p "$HOME/.local/bin"
  curl -fsSL https://dl.dagger.io/dagger/install.sh | BIN_DIR="$HOME/.local/bin" sh
}

install_pwsh() {
  log "PowerShell 7 (Linux)"
  if is_linux_bin pwsh; then
    return 0
  fi
  need_sudo
  # shellcheck disable=SC1091
  source /etc/os-release
  local deb
  deb="$(mktemp --suffix=.deb)"
  if curl -fsSL "https://packages.microsoft.com/config/ubuntu/${VERSION_ID}/packages-microsoft-prod.deb" -o "$deb"; then
    sudo dpkg -i "$deb" || true
    sudo apt-get update -y
    if apt-cache show powershell >/dev/null 2>&1; then
      sudo DEBIAN_FRONTEND=noninteractive apt-get install -y powershell
      rm -f "$deb"
      return 0
    fi
  fi
  rm -f "$deb"
  log "powershell package not in Microsoft apt for ${VERSION_ID}; installing GitHub tarball"
  local tag tmp
  tag="$(curl -fsSL https://api.github.com/repos/PowerShell/PowerShell/releases/latest | sed -n 's/.*"tag_name": "\(v[^"]*\)".*/\1/p' | head -n1)"
  tmp="$(mktemp -d)"
  curl -fsSL "https://github.com/PowerShell/PowerShell/releases/download/${tag}/powershell-${tag#v}-linux-x64.tar.gz" -o "$tmp/pwsh.tgz"
  sudo mkdir -p /opt/microsoft/powershell/7
  sudo tar -C /opt/microsoft/powershell/7 -xzf "$tmp/pwsh.tgz"
  sudo chmod +x /opt/microsoft/powershell/7/pwsh
  sudo ln -sfn /opt/microsoft/powershell/7/pwsh /usr/local/bin/pwsh
  rm -rf "$tmp"
}

install_starship() {
  log "starship"
  if ! is_linux_bin starship; then
    curl -sS https://starship.rs/install.sh | sh -s -- -y -b "$HOME/.local/bin"
  fi
}

install_zoxide() {
  log "zoxide"
  if ! is_linux_bin zoxide; then
    curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh
  fi
}

install_atuin() {
  log "atuin"
  if ! is_linux_bin atuin; then
    curl --proto '=https' --tlsv1.2 -LsSf https://setup.atuin.sh | sh -s -- --non-interactive
  fi
  # Official installer puts the binary in ~/.atuin/bin
  export PATH="$HOME/.atuin/bin:$PATH"
}

install_opencode() {
  log "opencode"
  if ! is_linux_bin opencode; then
    curl -fsSL https://opencode.ai/install | bash
  fi
}

install_gh() {
  log "gh"
  if is_linux_bin gh; then
    return 0
  fi
  local tag ver tmp
  tag="$(curl -fsSL https://api.github.com/repos/cli/cli/releases/latest | sed -n 's/.*"tag_name": "\(v[^"]*\)".*/\1/p' | head -n1)"
  ver="${tag#v}"
  tmp="$(mktemp -d)"
  curl -fsSL "https://github.com/cli/cli/releases/download/${tag}/gh_${ver}_linux_amd64.tar.gz" -o "$tmp/gh.tgz"
  tar -xzf "$tmp/gh.tgz" -C "$tmp"
  install -m 0755 "$tmp/gh_${ver}_linux_amd64/bin/gh" "$HOME/.local/bin/gh"
  rm -rf "$tmp"
}

install_claude() {
  log "claude code"
  if ! have claude; then
    curl -fsSL https://claude.ai/install.sh | bash
  fi
}

install_grok() {
  log "grok"
  if [ ! -x "$HOME/.grok/bin/grok" ]; then
    curl -fsSL https://x.ai/cli/install.sh | bash
  fi
}

install_op() {
  log "1Password CLI (op)"
  if [ -x "$HOME/.local/bin/op" ] || have op; then
    return 0
  fi
  need_sudo
  if [ ! -f /usr/share/keyrings/1password-archive-keyring.gpg ]; then
    curl -sS https://downloads.1password.com/linux/keys/1password.asc \
      | sudo gpg --dearmor --output /usr/share/keyrings/1password-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/1password-archive-keyring.gpg] https://downloads.1password.com/linux/debian/$(dpkg --print-architecture) stable main" \
      | sudo tee /etc/apt/sources.list.d/1password.list >/dev/null
  fi
  sudo apt-get update -y
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y 1password-cli
}

print_summary() {
  log "versions"
  printf 'git        %s\n' "$(git --version 2>/dev/null || echo missing)"
  printf 'gh         %s\n' "$(gh --version 2>/dev/null | head -n1 || echo missing)"
  printf 'pwsh       %s\n' "$(pwsh --version 2>/dev/null || echo missing)"
  printf 'docker     %s\n' "$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo 'start Docker Desktop')"
  printf 'node       %s\n' "$(node --version 2>/dev/null || echo missing)"
  printf 'bun        %s\n' "$(bun --version 2>/dev/null || echo missing)"
  printf 'go         %s\n' "$(go version 2>/dev/null || echo missing)"
  printf 'dotnet     %s\n' "$(dotnet --version 2>/dev/null || echo missing)"
  printf 'python     %s\n' "$(python3 --version 2>/dev/null || echo missing)"
  printf 'rustc      %s\n' "$(rustc --version 2>/dev/null || echo missing)"
  printf 'op         %s\n' "$(op --version 2>/dev/null || echo missing)"
  printf 'dagger     %s\n' "$(dagger version 2>/dev/null | head -n1 || echo missing)"
  printf 'starship   %s\n' "$(starship --version 2>/dev/null | head -n1 || echo missing)"
  printf 'zoxide     %s\n' "$(zoxide --version 2>/dev/null || echo missing)"
  printf 'fzf        %s\n' "$(fzf --version 2>/dev/null || echo missing)"
  printf 'atuin      %s\n' "$(atuin --version 2>/dev/null || echo missing)"
  printf 'opencode   %s\n' "$(opencode --version 2>/dev/null || echo missing)"
  printf 'claude     %s\n' "$(claude --version 2>/dev/null || echo missing)"
  printf 'grok       %s\n' "$(grok --version 2>/dev/null || echo missing)"
  printf 'oh-my-posh %s\n' "$(command -v oh-my-posh >/dev/null && echo 'STILL PRESENT (should be Windows-only)' || echo 'not in WSL (ok)')"
}

main() {
  mkdir -p "$HOME/.local/bin" "$HOME/code"
  remove_oh_my_posh
  ensure_bashrc
  install_apt
  install_uv_python
  install_rust
  install_bun
  install_fnm_node
  install_go
  install_dotnet
  install_gh
  install_dagger
  install_pwsh
  install_starship
  install_zoxide
  install_atuin
  install_opencode
  install_claude
  install_grok
  install_op
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  print_summary
  echo
  echo "Prompt in WSL is Starship. Oh My Posh stays on Windows only."
  echo "VS Code stays on Windows. From a Linux path:  code ."
}

main "$@"
