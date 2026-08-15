# WSL Ubuntu workstation

Idempotent bootstrap for a fresh Ubuntu WSL distro. Re-run `./install.sh` any time.

Ubuntu has **no winget**. Closest tools:

| Need | Use |
|---|---|
| System libraries, `git`, `jq`, audio utils | `apt` (this repo's `packages/apt.txt`) |
| Node / Python / Rust / Go versions | official installers or `uv` / `fnm` / `rustup` (this script) |
| One command for lots of user CLIs | [Linux Homebrew](https://docs.brew.sh/Homebrew-on-Linux) + a `Brewfile` |
| Pin every runtime in one place | [mise](https://mise.jdx.dev/) |

This repo uses **apt + official installers**. That matches how Claude, Grok, Bun, Go, and .NET actually want to be installed.

## Fresh distro

```bash
sudo apt update && sudo apt install -y git curl
git clone git@github.com:pjmagee/wsl-setup.git ~/code/wsl-setup
cd ~/code/wsl-setup
./install.sh
```

Then open a new Ubuntu tab so `PATH` picks up rustup/fnm/uv.

## What it installs

| Wanted | How | Notes |
|---|---|---|
| `git` | apt | |
| `gh` | GitHub release tarball | `gh auth login` still needed for the API |
| `bun` | bun.sh | |
| `dotnet` | Microsoft `dotnet-install.sh` → `~/.dotnet` | SDK 10 |
| `go` | official tarball → `~/.local/go` | |
| `python3.14` | `uv python install 3.14` | Ubuntu 26.04 apt `python3` is already 3.14 |
| `rustc` / `cargo` | rustup stable | |
| `op` | existing `~/.local/bin/op`, else 1Password apt | Linux `op` does **not** get Windows Hello |
| `claude` | `claude.ai/install.sh` | |
| `grok` | `x.ai/cli/install.sh` | |
| `code` | **not installed here** | Windows VS Code + `code .` (Remote-WSL) |
| `docker` | **not installed here** | Docker Desktop WSL integration |

## Keep on Windows

- VS Code / Cursor / JetBrains UI
- Discord, browsers, Steam, 1Password desktop
- Docker Desktop

## Useful extras this script adds

`build-essential`, `ripgrep`, `fd`, `fzf`, `jq`, `tmux`, `htop`, `shellcheck`, `libicu-dev`, `pulseaudio-utils` (for diagnosing WSLg audio).

You already have `dagger`, `kubectl` (from Docker Desktop), and `oh-my-posh` in `~/.local/bin`. Add them to `install.sh` if you want them on the next machine.
