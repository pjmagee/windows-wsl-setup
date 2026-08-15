# wsl-setup

Idempotent bootstrap for a **WSL 2 Ubuntu 26.04** development machine.

This repo is the source of truth for the toolchain. Clone it on a **clean Windows host** (a work laptop with no winget / UniGetUI copies of these CLIs) and you still get every tool. Nothing here depends on Windows interop versions of `az`, `gcloud`, `aws`, Node, Git, etc.

Safe to re-run.

## Requirements (Windows host only)

The host OS does **not** need the developer CLIs. It only needs:

- Windows 11 with WSL 2
- Ubuntu **26.04 LTS** (`wsl --install Ubuntu-26.04`)
- `sudo` inside the distro (passwordless is convenient)
- [Docker Desktop](https://docs.docker.com/desktop/features/wsl/) with WSL integration enabled for this distro (needed for `docker` / Dagger)
- [Visual Studio Code](https://code.visualstudio.com/) on **Windows**, with the [WSL](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-wsl) extension

## Install

```bash
sudo apt update && sudo apt install -y git curl
git clone https://github.com/pjmagee/wsl-setup.git ~/code/wsl-setup
cd ~/code/wsl-setup
./install.sh
```

Open a new Ubuntu tab so `~/.bashrc` loads.

Keep repositories on the **Linux** disk (`~/code/...`), not `/mnt/c` or `/mnt/d`:

```bash
cd ~/code/your-repo
code .
```

## What it installs

| Tool | Source |
|---|---|
| System packages | `apt` via [`packages/apt.txt`](packages/apt.txt) — compilers, `git`, `jq`, `ripgrep`, `fd`, `fzf`, `tmux`, `wl-clipboard`, ICU |
| Node.js (LTS) | [fnm](https://github.com/Schniz/fnm) |
| bun | [bun.sh](https://bun.sh) |
| Go | official tarball → `~/.local/go` |
| .NET SDK 10 | Microsoft `dotnet-install.sh` → `~/.dotnet` |
| Python 3.14 | [uv](https://docs.astral.sh/uv/) |
| Rust | rustup (stable) |
| PowerShell 7 | Microsoft apt repo, or GitHub tarball if that repo is missing |
| GitHub CLI | official Linux release (`gh auth login` once) |
| Dagger | official **Linux** installer → `~/.local/bin/dagger` |
| Starship | official installer (WSL prompt) |
| zoxide | official installer (`z` jump) |
| fzf | Ubuntu apt + bash keybindings |
| atuin | official installer (shell history) |
| OpenCode | [opencode.ai](https://opencode.ai/docs/) install script |
| 1Password CLI (`op`) | 1Password apt repo (no Windows Hello) |
| Claude Code | official installer |
| Grok Build | official installer |
| Azure CLI (`az`) | Microsoft azure-cli apt repo (`noble` fallback until 26.04 is published) |
| Azure Developer CLI (`azd`) | official `install-azd.sh` |
| Google Cloud CLI (`gcloud`) | official `google-cloud-cli` apt repo |
| saml2aws | latest GitHub release → `~/.local/bin` |
| AWS CLI v2 (`aws`) | official Linux installer → `/usr/local/bin/aws` |
| Cloudflare CLI (`cf`) | npm `cf@latest` (current Cloudflare CLI) |
| Wrangler | npm `wrangler@latest` (Workers / Pages) |
| cloudflared | official Cloudflare apt repo |
| MongoDB Compass | official Linux `.deb` + WSLg wrapper (`compass`) |

**Linux GUIs (WSLg)**

MongoDB Compass is the **Linux** app. From Ubuntu:

```bash
compass
# or
mongodb-compass
```

WSLg puts the window on the Windows desktop. Requires `guiApplications=true` in `.wslconfig` (the Windows 11 default).

Electron may print a `StartTransientUnit` / `app-MongoDB Compass-….scope` D-Bus line. That is a known Chromium bug (a space in the app name). The window still opens; the `compass` wrapper hides it.

**Keep on Windows**

- VS Code / Cursor / JetBrains UI — `code .` from a Linux path
- Docker Engine — Docker Desktop WSL integration
- Oh My Posh — Windows Terminal / PowerShell only
- Discord, browsers, 1Password desktop, games

## Updates

```bash
cd ~/code/wsl-setup && git pull && ./install.sh
```

Or by layer:

```bash
sudo apt update && sudo apt full-upgrade
bun upgrade
rustup update
fnm install --lts
uv python install 3.14
```

WSL kernel / WSLg: `wsl --update` from Windows.

## Design

- Ubuntu **26.04 only**.
- Built for a work laptop whose Windows host has **no** interop copies of these tools.
- `install.sh` ignores Windows binaries on `PATH` (`/mnt/c/...`, `*.exe`) and installs Linux ones.
- Linux binaries are prepended on `PATH` so WSL interop cannot win even if the host later grows winget packages.
- Language runtimes come from upstream installers, not stale `apt` packages.
- Edit [`packages/apt.txt`](packages/apt.txt) to add system packages.

## License

Use and fork as you like.
