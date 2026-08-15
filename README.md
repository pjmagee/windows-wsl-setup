# wsl-setup

Idempotent bootstrap for a **WSL 2 Ubuntu** development machine.

It installs a Linux toolchain *inside* the distro (so tools are not the Windows copies on `PATH`) and is safe to re-run.

## Requirements

- Windows 11 with WSL 2
- An Ubuntu distro (`Ubuntu-24.04` or `Ubuntu-26.04`)
- `sudo` (passwordless is convenient)
- [Docker Desktop](https://docs.docker.com/desktop/features/wsl/) with WSL integration enabled for this distro (optional, but needed for `docker` / Dagger)
- [Visual Studio Code](https://code.visualstudio.com/) on **Windows**, with the [WSL](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-wsl) extension

## Install

```bash
sudo apt update && sudo apt install -y git curl
git clone https://github.com/pjmagee/wsl-setup.git ~/code/wsl-setup
cd ~/code/wsl-setup
./install.sh
```

Open a new Ubuntu tab so `~/.bashrc` picks up `PATH` and Oh My Posh.

Put project checkouts on the **Linux** disk (`~/code/...`), not under `/mnt/c` or `/mnt/d`. Then:

```bash
cd ~/code/your-repo
code .
```

## What it installs

| Tool | Source | Notes |
|---|---|---|
| System packages | `apt` via [`packages/apt.txt`](packages/apt.txt) | compilers, `git`, `jq`, `ripgrep`, `fd`, `fzf`, `tmux`, ICU, PulseAudio utils |
| Node.js (LTS) | [fnm](https://github.com/Schniz/fnm) | not the Windows `node.exe` |
| bun | [bun.sh](https://bun.sh) | |
| Go | official tarball → `~/.local/go` | |
| .NET SDK 10 | Microsoft `dotnet-install.sh` → `~/.dotnet` | |
| Python 3.14 | [uv](https://docs.astral.sh/uv/) | also a `python3.14` shim |
| Rust | rustup (stable) | |
| GitHub CLI | official Linux release | run `gh auth login` once |
| Dagger | official Linux installer → `~/.local/bin/dagger` | **not** the Windows binary |
| Oh My Posh | official installer | inits from `~/.bashrc`; use a Nerd Font in Windows Terminal |
| 1Password CLI (`op`) | 1Password apt repo | Linux `op` does not use Windows Hello |
| Claude Code | official installer | |
| Grok Build | official installer | |

**Not installed here** (keep these on Windows):

- VS Code / Cursor / JetBrains UI — use `code .` from a Linux path
- Docker Engine — use Docker Desktop’s WSL integration
- Discord, browsers, 1Password desktop, games

## Updates

Re-run the script at any time:

```bash
cd ~/code/wsl-setup && git pull && ./install.sh
```

Or update layers yourself:

```bash
sudo apt update && sudo apt full-upgrade
bun upgrade
rustup update
fnm install --lts
uv python install 3.14
```

WSL itself (kernel / WSLg) is updated from Windows: `wsl --update`.

## Design

- Linux binaries are prepended on `PATH` so WSL interop does not pick `*.exe` from Windows.
- Language runtimes come from upstream installers, not stale `apt` packages.
- `packages/apt.txt` is the only list to edit for extra system packages. Unknown names are skipped (so the same file works on 24.04 and 26.04).

## License

Use and fork as you like.
