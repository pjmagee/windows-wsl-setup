# wsl-setup

Idempotent bootstrap for a **WSL 2 Ubuntu 26.04** development machine.

This repo is the source of truth for the toolchain. Clone it on a **clean Windows host** (a work laptop with no winget / UniGetUI copies of these CLIs) and you still get every tool. Nothing here depends on Windows interop versions of `az`, `gcloud`, `aws`, Node, Git, etc.

Safe to re-run.

## Requirements (Windows host only)

The host OS does **not** need the developer CLIs. It only needs:

- Windows 11 with WSL 2 (`wsl --version`)
- Ubuntu **26.04 LTS** (`wsl --install Ubuntu-26.04`)
- `guiApplications=true` under `[wsl2]` in `%USERPROFILE%\.wslconfig` (Windows 11 default; this is WSLg)
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

`wsl` from PowerShell starts in the Windows cwd (`/mnt/c/Users/...`, 9P). That is slow; Starship may warn that directory scan timed out. Use the **Ubuntu 26.04** Windows Terminal profile (`wsl.exe -d Ubuntu-26.04 --cd ~`) or:

```powershell
wsl ~
```

Keep repositories on the **Linux** disk (`~/code/...`), not `/mnt/c` or `/mnt/d`:

```bash
cd ~/code/your-repo
code .
```

## What it installs

| Tool | Source |
|---|---|
| System packages | `apt` via [`packages/apt.txt`](packages/apt.txt) — compilers, `git`, `jq`, `ripgrep`, `fd`, `fzf`, `tmux`, `wl-clipboard`, ICU, fonts |
| Node.js (LTS) | [fnm](https://github.com/Schniz/fnm) |
| bun | [bun.sh](https://bun.sh) |
| Go | official tarball → `~/.local/go` |
| .NET SDK 10 | Microsoft `dotnet-install.sh` → `~/.dotnet` |
| Python 3.14 | [uv](https://docs.astral.sh/uv/) |
| Rust | rustup (stable) |
| PowerShell 7 | Microsoft apt repo, or GitHub tarball if that repo is missing |
| GitHub CLI | official Linux release (`gh auth login` once) |
| Dagger | official **Linux** installer → `~/.local/bin/dagger` |
| Starship | official installer + [`starship.toml`](starship.toml) (`scan_timeout` for `/mnt/c`) |
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
| Cloudflare CLI (`cf`) | npm `cf@latest` |
| Wrangler | npm `wrangler@latest` |
| cloudflared | official Cloudflare apt repo |
| MongoDB Compass | official Linux `.deb`; `compass` launches it through WSLg |
| `wsl-open` | [`scripts/wsl-open`](scripts/wsl-open) — Linux `http`/`https`/`mailto` → Windows default browser |

## Linux GUIs (WSLg)

WSLg (`guiApplications=true`) remotes Linux windows onto the Windows desktop.

```bash
xclock          # sanity check
compass         # Linux MongoDB Compass
```

The window title must be the **app name**. `[WARN: COPY MODE]` means WSLg fell back to copying pixels over RDP instead of shared memory (slow, some apps do not paint). Fix:

```powershell
wsl --shutdown
```

Then open Ubuntu again and retry `xclock`. If COPY MODE persists, `wsl --update` from Windows and another shutdown.

`compass` starts the **Linux** package (`/usr/bin/mongodb-compass`). The optional Windows EXE:

```bash
COMPASS_WINDOWS=1 compass
```

CUDA on the NVIDIA GPU works in WSL (`nvidia-smi`). Compass does **not** use that path for its UI (Electron 41 + Mesa D3D12 does not map a window). The wrapper uses software raster (SwiftShader); WSLg still displays it on Windows.

## Browser links

Ubuntu 26.04 **removed** `wslu` / `wslview` from the archive ([LP #2131669](https://bugs.launchpad.net/ubuntu/+source/wslu/+bug/2131669): discontinued upstream). This repo does not reinstall it.

`wsl-open` is set as `BROWSER` and `GH_BROWSER`. Clicks from `gh`, `az login`, Compass, and similar open in the Windows default browser.

```bash
wsl-open https://example.com
```

## Keep on Windows

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
