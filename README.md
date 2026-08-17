# wsl-setup

Idempotent bootstrap for a **WSL 2 Ubuntu 26.04** development machine.

This repo is the source of truth for the toolchain. Clone it on a **clean Windows host** (a work laptop with no winget / UniGetUI copies of these CLIs) and you still get every tool. Nothing here depends on Windows interop versions of `az`, `gcloud`, `aws`, Node, Git, etc.

Safe to re-run.

On a **new Windows 11 work laptop**, clone this repo, open it in VS Code, and ask Copilot:

> Read AGENTS.md and set up WSL on this Windows 11 laptop from this repo.

[`AGENTS.md`](AGENTS.md) is the agent playbook (Copilot at work; Grok or Claude at home).

## Requirements (Windows host only)

The host OS does **not** need the developer CLIs. It only needs:

- Windows 11 with WSL 2 (`wsl --version`)
- Ubuntu **26.04 LTS** (`wsl --install Ubuntu-26.04` if it is not already there)
- `guiApplications=true` under `[wsl2]` in `%USERPROFILE%\.wslconfig` (Windows 11 default; this is WSLg)
- Passwordless `sudo` inside the distro (`sudo -n` — `install.sh` will not prompt)
- [Docker Desktop](https://docs.docker.com/desktop/features/wsl/) with WSL integration enabled for **Ubuntu-26.04** (needed for `docker` / Dagger; flip this if it still targets 24.04)
- [Visual Studio Code](https://code.visualstudio.com/) on **Windows**, with the [WSL](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-wsl) extension

## Install

### From Windows (work laptop / first time)

```powershell
git clone https://github.com/pjmagee/wsl-setup.git
cd wsl-setup
powershell -NoProfile -ExecutionPolicy Bypass -File .\windows\bootstrap.ps1
```

Reboot if Windows asks, then run the same command again.

`bootstrap.ps1` is for a work laptop that **already exists**. It updates WSL if it can (26.04 wants WSL 2.4.10+), installs Ubuntu 26.04 only if missing (normal WSL username/password prompt once, then auto sign-in), turns on NOPASSWD sudo for that existing user, makes `wsl` and `ubuntu` open the distro at `~` in Windows Terminal, copies this repo to `~/code/wsl-setup` on the Linux disk, and runs `install.sh work` (Copilot CLI; no grok/claude/opencode/devtunnel/changie/hugo/stripe). It does not use cloud-init. It does **not** unregister leftover distros (`Ubuntu-24.04`, Store `Ubuntu`, `docker-desktop`).

If a **required** host step fails (passwordless sudo, `wsl-open`), the host script exits with an error — it will not print `Done.` A blocked vendor domain skips that tool and the run continues; the summary lists what to retry.

An old Store `ubuntu.exe` can still win over `ubuntu.cmd` in **cmd** (`PATHEXT` prefers `.EXE`). Use the new Windows Terminal **Ubuntu** profile, or PowerShell's `ubuntu` function. Point Docker Desktop's WSL integration at **Ubuntu-26.04** if it is still attached to 24.04.

### Already inside Ubuntu 26.04

```bash
sudo apt update && sudo apt install -y git curl
git clone https://github.com/pjmagee/wsl-setup.git ~/code/wsl-setup
cd ~/code/wsl-setup
./install.sh work    # work laptop (Copilot)
# ./install.sh home  # home machine (Grok / Claude)
```

`sudo -n` must already work. If it does not, run `windows/bootstrap.ps1` from Windows first.

The profile is saved in `~/.config/wsl-setup/profile`. Bare `./install.sh` reuses it.

Open a new Ubuntu tab so `~/.bashrc` loads.

## Windows Terminal

After bootstrap, a **new** Windows Terminal window:

- Default profile is **Ubuntu**, starting in the Linux home (`~`), not `/mnt/c`
- Profiles **Ubuntu** and **wsl** are the same session
- `ubuntu` in PowerShell and a bare `wsl` (new PowerShell session) also land at `~`
- `ubuntu` in **cmd** may still be Store `ubuntu.exe` (24.04) if that alias exists; use the Terminal profile

Plain `wsl.exe` with no args still inherits the Windows cwd (`/mnt/c/Users/...`, 9P). That is slow; Starship may warn that directory scan timed out. Use the Terminal profile, or:

```powershell
wsl -d Ubuntu-26.04 ~
```

Keep repositories on the **Linux** disk (`~/code/...`), not `/mnt/c` or `/mnt/d`:

```bash
cd ~/code/your-repo
code .
```

## Profiles

One installer, three profiles. Step lists live in [`profiles/`](profiles/) so agents can read them without parsing bash.

| Profile | Command | What you get |
|---|---|---|
| **universal** | `./install.sh universal` | Shared toolchain only ([`profiles/universal.txt`](profiles/universal.txt)) |
| **work** | `./install.sh work` | Universal + GitHub Copilot CLI. **Not** grok, claude, opencode, devtunnel, changie, hugo, stripe. `bootstrap.ps1` uses this. |
| **home** | `./install.sh home` | Universal + grok, claude, opencode, devtunnel, changie, hugo, stripe |

`install.sh work` also **removes** those home-only binaries if a previous full run left them behind.

## What it installs

| Tool | Source |
|---|---|
| System packages | `apt` via [`packages/apt.txt`](packages/apt.txt) — compilers, `git`, `jq`, `ripgrep`, `fd`, `fzf`, `tmux`, `wl-clipboard`, ICU, fonts |
| Node.js (LTS) | [fnm](https://github.com/Schniz/fnm) |
| bun | [bun.sh](https://bun.sh) |
| Go | official tarball → `~/.local/go` |
| .NET SDK 10 | Microsoft `dotnet-install.sh` → `~/.dotnet` |
| Python 3.14 | [uv](https://docs.astral.sh/uv/) (official `install.sh`; `uv` CLI stays on PATH) |
| Rust | rustup (stable) |
| PowerShell 7 | Microsoft apt repo, or GitHub tarball if that repo is missing |
| GitHub CLI | official Linux release (`gh auth login` once) |
| Dagger | official **Linux** installer → `~/.local/bin/dagger` |
| Starship | official installer + [`starship.toml`](starship.toml) (`scan_timeout` for `/mnt/c`) |
| zoxide | official installer (`z` jump) |
| fzf | Ubuntu apt + bash keybindings |
| atuin | official installer (shell history) |
| OpenCode | [opencode.ai](https://opencode.ai/docs/) install script (**home** profile) |
| 1Password CLI (`op`) | 1Password apt repo (no Windows Hello) |
| 1Password SSH | `ssh` / `ssh-add` aliases → Windows `ssh.exe`; `git` `core.sshCommand=ssh.exe` |
| Claude Code | official installer (**home** profile) |
| GitHub Copilot CLI (`copilot`) | official `gh.io/copilot-install`, else GitHub tarball, else npm (**work** profile) |
| Grok Build | official installer (**home** profile) |
| Azure CLI (`az`) | Microsoft azure-cli apt repo (`noble` fallback until 26.04 is published) |
| Azure Developer CLI (`azd`) | official `install-azd.sh` |
| Google Cloud CLI (`gcloud`) | official `google-cloud-cli` apt repo |
| saml2aws | latest GitHub release → `~/.local/bin` |
| AWS CLI v2 (`aws`) | official Linux installer → `/usr/local/bin/aws` |
| Cloudflare CLI (`cf`) | npm `cf@latest` |
| Wrangler | npm `wrangler@latest` |
| cloudflared | official Cloudflare apt repo |
| MongoDB Compass | official Linux `.deb`; `compass` launches it through WSLg |
| Microsoft Dev Tunnels (`devtunnel`) | official Linux install script (**home** profile) |
| changie | latest GitHub release → `~/.local/bin` (**home** profile) |
| Helm | official `get-helm-4` script → `~/.local/bin` |
| Hugo (extended) | official GitHub tarball → `~/.local/bin` (**home** profile) |
| Stripe CLI | official Stripe apt repo (**home** profile) |
| 7-Zip (`7zz`) | official Linux tarball → `~/.local/bin` (`7z` symlink) |
| MongoDB Shell (`mongosh`) | official Linux tarball → `~/.local/opt/mongosh` |
| Flux CD (`flux`) | official `fluxcd.io/install.sh` → `~/.local/bin` |
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
- Discord, browsers, 1Password desktop + SSH agent, games
- Python 3.14, [uv](https://docs.astral.sh/uv/), and [atuin](https://atuin.sh/) (host copies; the Linux ones still come from `install.sh`)

## 1Password SSH

The 1Password SSH agent runs on **Windows**. WSL reaches it by calling Microsoft OpenSSH (`ssh.exe`), not Linux `ssh`.

`install.sh` writes this to `~/.bash_aliases` and `~/.zshrc` (idempotent, marked block):

```bash
alias ssh='ssh.exe'
alias ssh-add='ssh-add.exe'
```

and sets:

```bash
git config --global core.sshCommand ssh.exe
```

Requirements on the Windows host (not installed by this repo):

- 1Password for Windows signed in, **Use the SSH agent** enabled
- Windows **OpenSSH Authentication Agent** service disabled (1Password owns `\\.\pipe\openssh-ssh-agent`)
- At least one SSH key in 1Password available to the agent

SSH host config belongs in `%USERPROFILE%\.ssh\config`, not WSL `~/.ssh/config`. Commit signing is configured from the 1Password app (**Configure Commit Signing** → **Configure for WSL**).

```bash
ssh-add -l                 # lists 1Password keys
ssh -T git@github.com      # should prompt 1Password, then authenticate
```

## Updates

```bash
cd ~/code/wsl-setup && git pull && ./install.sh          # saved profile
# ./install.sh home
# ./install.sh work
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
- NOPASSWD sudo for the existing WSL user (`windows/bootstrap.ps1` / `ensure-user.sh`).
- Built for a work laptop whose Windows host has **no** interop copies of these tools.
- `install.sh` ignores Windows binaries on `PATH` (`/mnt/c/...`, `*.exe`) and installs Linux ones.
- Linux binaries are prepended on `PATH` so WSL interop cannot win even if the host later grows winget packages.
- Language runtimes come from upstream installers, not stale `apt` packages.
- Profiles split the toolchain: [`profiles/universal.txt`](profiles/universal.txt) is shared; [`profiles/work.txt`](profiles/work.txt) adds Copilot; [`profiles/home.txt`](profiles/home.txt) adds the home extras.
- Optional tools are skipped when their host is blocked or the installer errors (`run_step`). Re-run `./install.sh` later. Curl and apt use short timeouts so a dead repo cannot hang the run.
- Edit [`packages/apt.txt`](packages/apt.txt) to add system packages.

## License

Use and fork as you like.
