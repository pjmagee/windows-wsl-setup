# wsl-setup

Idempotent bootstrap for a **WSL 2 Ubuntu 26.04** development machine.

It installs a Linux toolchain *inside* the distro so Windows copies on `PATH` are not used. Safe to re-run.

## Requirements

- Windows 11 with WSL 2
- Ubuntu **26.04 LTS** (`wsl --install Ubuntu-26.04`)
- `sudo` (passwordless is convenient)
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
- Linux binaries are prepended on `PATH` so WSL interop does not pick `*.exe`.
- Language runtimes come from upstream installers, not stale `apt` packages.
- Edit [`packages/apt.txt`](packages/apt.txt) to add system packages.

## License

Use and fork as you like.
