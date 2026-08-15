#!/usr/bin/env bash
# Idempotent Ubuntu WSL workstation bootstrap.
# Safe to re-run. Does not install Linux VS Code or Discord.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export HOME="${HOME:-/home/magaoidh}"
export PATH="$HOME/.local/bin:$HOME/.bun/bin:$HOME/.dotnet:$HOME/.dotnet/tools:$HOME/.local/go/bin:$HOME/go/bin:$HOME/.cargo/bin:$HOME/.grok/bin:$PATH"

log() { printf '\n==> %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

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

ensure_bashrc_path() {
  local marker=">>> wsl-linux-path >>>"
  if ! grep -q "$marker" "$HOME/.bashrc" 2>/dev/null; then
    log "note: $HOME/.bashrc is missing the Linux-first PATH block; not rewriting it"
  fi
}

install_uv_python() {
  log "uv + Python 3.14"
  if ! have uv; then
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
  fi
  uv python install 3.14
  # User-level shim so `python3.14` exists on PATH
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

install_gh() {
  log "gh"
  if have gh && [[ "$(command -v gh)" != /mnt/c/* ]]; then
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
  # Keep the existing user-level binary if present. Official apt package
  # is Linux-native and will NOT unlock via Windows Hello.
  if [ -x "$HOME/.local/bin/op" ]; then
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
  printf 'git      %s\n' "$(git --version 2>/dev/null || echo missing)"
  printf 'gh       %s\n' "$(gh --version 2>/dev/null | head -n1 || echo missing)"
  printf 'docker   %s\n' "$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo 'start Docker Desktop')"
  printf 'node     %s\n' "$(node --version 2>/dev/null || echo missing)"
  printf 'bun      %s\n' "$(bun --version 2>/dev/null || echo missing)"
  printf 'go       %s\n' "$(go version 2>/dev/null || echo missing)"
  printf 'dotnet   %s\n' "$(dotnet --version 2>/dev/null || echo missing)"
  printf 'python   %s\n' "$(python3 --version 2>/dev/null || echo missing)"
  printf 'python3.14 %s\n' "$(python3.14 --version 2>/dev/null || echo missing)"
  printf 'rustc    %s\n' "$(rustc --version 2>/dev/null || echo missing)"
  printf 'op       %s\n' "$(op --version 2>/dev/null || echo missing)"
  printf 'claude   %s\n' "$(claude --version 2>/dev/null || echo missing)"
  printf 'grok     %s\n' "$(grok --version 2>/dev/null || echo missing)"
  printf 'code     %s\n' "$(code --version 2>/dev/null | head -n1 || echo 'install VS Code on Windows')"
}

main() {
  mkdir -p "$HOME/.local/bin" "$HOME/code"
  ensure_bashrc_path
  install_apt
  install_uv_python
  install_rust
  install_bun
  install_fnm_node
  install_go
  install_dotnet
  install_gh
  install_claude
  install_grok
  install_op
  # Refresh cargo env for the summary
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  print_summary
  echo
  echo "VS Code stays on Windows. From a Linux path:  code ."
  echo "Do not install Discord inside WSL. Use the Windows app."
}

main "$@"
