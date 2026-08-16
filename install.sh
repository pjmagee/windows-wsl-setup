#!/usr/bin/env bash
# Idempotent Ubuntu 26.04 WSL workstation bootstrap.
# Installs native Linux tools only. Does not use Windows interop copies.
# Safe to re-run. Does not install Linux VS Code, Discord, or Oh My Posh.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${HOME:=$(getent passwd "$(id -un)" | cut -d: -f6)}"
export HOME
export PATH="$HOME/.local/bin:$HOME/.atuin/bin:$HOME/.opencode/bin:$HOME/.bun/bin:$HOME/.dotnet:$HOME/.dotnet/tools:$HOME/.local/go/bin:$HOME/go/bin:$HOME/.cargo/bin:$HOME/.grok/bin:$PATH"

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

  local path_block tmp stripped
  path_block="$(mktemp)"
  tmp="$(mktemp)"
  stripped="$(mktemp)"
  cat >"$path_block" <<'EOF'
# >>> wsl-linux-path >>>
# Linux toolchains must precede the Windows PATH that WSL appends.
# Do not rely on winget / UniGetUI / Windows copies of these CLIs.
export PATH="$HOME/.local/bin:$HOME/.atuin/bin:$HOME/.opencode/bin:$HOME/.bun/bin:$HOME/.dotnet:$HOME/.dotnet/tools:$HOME/.local/go/bin:$HOME/go/bin:$HOME/.cargo/bin:$HOME/.grok/bin:$PATH"
if [ -d "$HOME/.local/share/fnm" ]; then
  export PATH="$HOME/.local/share/fnm:$PATH"
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

install_uv_python() {
  log "uv + Python 3.14"
  if ! is_linux_bin uv; then
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
  if ! is_linux_bin rustup; then
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
  if [ ! -x "$HOME/.local/share/fnm/fnm" ] && ! is_linux_bin fnm; then
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
  mkdir -p "$HOME/.config"
  if [ ! -f "$HOME/.config/starship.toml" ]; then
    install -m 0644 "$ROOT/starship.toml" "$HOME/.config/starship.toml"
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
  if ! is_linux_bin claude; then
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
  if is_linux_bin op; then
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

# Microsoft has no azure-cli suite for Ubuntu 26.04 (resolute) yet. Official
# guidance is to use an earlier published suite; noble is the closest LTS.
azure_cli_suite() {
  local candidate
  for candidate in "$(lsb_release -cs)" noble jammy; do
    if curl -fsI "https://packages.microsoft.com/repos/azure-cli/dists/${candidate}/Release" >/dev/null 2>&1; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

install_az() {
  log "Azure CLI (az)"
  if is_linux_bin az; then
    return 0
  fi
  need_sudo
  sudo mkdir -p /etc/apt/keyrings
  if [ ! -f /etc/apt/keyrings/microsoft.gpg ]; then
    curl -sLS https://packages.microsoft.com/keys/microsoft.asc \
      | gpg --dearmor | sudo tee /etc/apt/keyrings/microsoft.gpg >/dev/null
    sudo chmod go+r /etc/apt/keyrings/microsoft.gpg
  fi
  local suite
  if ! suite="$(azure_cli_suite)"; then
    log "no azure-cli apt suite published; installing with uv tool"
    uv tool install azure-cli
    return 0
  fi
  printf 'Types: deb\nURIs: https://packages.microsoft.com/repos/azure-cli/\nSuites: %s\nComponents: main\nArchitectures: %s\nSigned-by: /etc/apt/keyrings/microsoft.gpg\n' \
    "$suite" "$(dpkg --print-architecture)" \
    | sudo tee /etc/apt/sources.list.d/azure-cli.sources >/dev/null
  sudo apt-get update -y
  if ! sudo DEBIAN_FRONTEND=noninteractive apt-get install -y azure-cli; then
    log "azure-cli apt install failed; installing with uv tool"
    uv tool install azure-cli
  fi
}

install_azd() {
  log "Azure Developer CLI (azd)"
  if is_linux_bin azd; then
    return 0
  fi
  need_sudo
  curl -fsSL https://aka.ms/install-azd.sh | bash -s -- --no-telemetry
}

install_gcloud() {
  log "Google Cloud CLI (gcloud)"
  if is_linux_bin gcloud; then
    return 0
  fi
  need_sudo
  sudo mkdir -p /usr/share/keyrings
  if [ ! -f /usr/share/keyrings/cloud.google.gpg ]; then
    curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg \
      | sudo gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
  fi
  echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" \
    | sudo tee /etc/apt/sources.list.d/google-cloud-sdk.list >/dev/null
  sudo apt-get update -y
  sudo CLOUDSDK_SKIP_PY_COMPILATION=1 DEBIAN_FRONTEND=noninteractive apt-get install -y google-cloud-cli
}

install_saml2aws() {
  log "saml2aws"
  if is_linux_bin saml2aws; then
    return 0
  fi
  local ver tmp
  ver="$(curl -fsSL https://api.github.com/repos/Versent/saml2aws/releases/latest | sed -n 's/.*"tag_name": "v\([^"]*\)".*/\1/p' | head -n1)"
  if [ -z "$ver" ]; then
    echo "could not resolve latest saml2aws release" >&2
    return 1
  fi
  tmp="$(mktemp -d)"
  curl -fsSL "https://github.com/Versent/saml2aws/releases/download/v${ver}/saml2aws_${ver}_linux_amd64.tar.gz" -o "$tmp/s.tgz"
  tar -xzf "$tmp/s.tgz" -C "$tmp"
  install -m 0755 "$tmp/saml2aws" "$HOME/.local/bin/saml2aws"
  rm -rf "$tmp"
}

install_aws() {
  log "AWS CLI v2"
  if is_linux_bin aws; then
    return 0
  fi
  need_sudo
  local tmp
  tmp="$(mktemp -d)"
  curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "$tmp/awscliv2.zip"
  unzip -q "$tmp/awscliv2.zip" -d "$tmp"
  if [ -x /usr/local/bin/aws ]; then
    sudo "$tmp/aws/install" --update
  else
    sudo "$tmp/aws/install"
  fi
  rm -rf "$tmp"
}

install_cloudflare() {
  log "Cloudflare CLI (cf, wrangler, cloudflared)"
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  if ! is_linux_bin npm; then
    echo "Linux npm is required for the Cloudflare CLIs (install_fnm_node first)" >&2
    return 1
  fi
  if ! is_linux_bin cf; then
    npm install -g cf@latest
  fi
  if ! is_linux_bin wrangler; then
    npm install -g wrangler@latest
  fi
  if is_linux_bin cloudflared; then
    return 0
  fi
  need_sudo
  sudo mkdir -p --mode=0755 /usr/share/keyrings
  if [ ! -f /usr/share/keyrings/cloudflare-main.gpg ]; then
    curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
      | sudo tee /usr/share/keyrings/cloudflare-main.gpg >/dev/null
  fi
  echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main" \
    | sudo tee /etc/apt/sources.list.d/cloudflared.list >/dev/null
  sudo apt-get update -y
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y cloudflared
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

print_summary() {
  log "versions"
  printf 'git        %s\n' "$(git --version 2>/dev/null || echo missing)"
  printf 'gh         %s\n' "$(gh --version 2>/dev/null | head -n1 || echo missing)"
  printf 'pwsh       %s\n' "$(pwsh --version 2>/dev/null || echo missing)"
  if is_linux_bin docker; then
    printf 'docker     %s\n' "$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo 'installed, daemon not running')"
  else
    printf 'docker     %s\n' "missing — Docker Desktop on Windows + enable this distro"
  fi
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
  printf 'az         %s\n' "$(az version -o json 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["azure-cli"])' 2>/dev/null || echo missing)"
  printf 'azd        %s\n' "$(azd version 2>/dev/null | head -n1 || echo missing)"
  printf 'gcloud     %s\n' "$(gcloud --version 2>/dev/null | head -n1 || echo missing)"
  printf 'saml2aws   %s\n' "$(saml2aws --version 2>&1 || echo missing)"
  printf 'aws        %s\n' "$(aws --version 2>/dev/null || echo missing)"
  printf 'cf         %s\n' "$(cf --version 2>/dev/null || echo missing)"
  printf 'wrangler   %s\n' "$(wrangler --version 2>/dev/null || echo missing)"
  printf 'cloudflared %s\n' "$(cloudflared --version 2>/dev/null || echo missing)"
  printf 'compass    %s\n' "$(dpkg-query -W -f='${Version}' mongodb-compass 2>/dev/null || echo missing)"
  printf 'oh-my-posh %s\n' "$(command -v oh-my-posh >/dev/null && echo 'STILL PRESENT (should be Windows-only)' || echo 'not in WSL (ok)')"
  if grep -q "alias ssh='ssh.exe'" "$HOME/.bash_aliases" 2>/dev/null; then
    printf '1p-ssh     aliases -> ssh.exe\n'
  else
    printf '1p-ssh     aliases missing\n'
  fi
  printf 'git-ssh    %s\n' "$(git config --global --get core.sshCommand 2>/dev/null || echo unset)"
}

main() {
  mkdir -p "$HOME/.local/bin" "$HOME/code"
  remove_oh_my_posh
  ensure_bashrc
  install_apt
  ensure_1password_ssh
  install_wsl_open
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
  install_az
  install_azd
  install_gcloud
  install_saml2aws
  install_aws
  install_cloudflare
  install_compass
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
  export PATH="$HOME/.local/share/fnm:$PATH"
  eval "$(fnm env --shell bash 2>/dev/null)" || true
  print_summary
  echo
  echo "Prompt in WSL is Starship. Oh My Posh stays on Windows only."
  echo "VS Code stays on Windows. From a Linux path:  code ."
  echo "MongoDB Compass is a Linux GUI:  compass    (window appears on Windows via WSLg)"
}

main "$@"
