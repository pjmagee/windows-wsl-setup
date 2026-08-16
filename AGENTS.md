# AGENTS.md

Playbook for coding agents (GitHub Copilot at work; Grok or Claude at home).
Human overview: [README.md](README.md).

You are bootstrapping the **same WSL 2 Ubuntu 26.04 workstation** this repo
defines — typically a **clean Windows 11 work laptop** that has none of the
home toolchain. Clone this repo, then **execute** the playbook. Do not invent
a second Linux installer.

At work, **Copilot is the operator**. Do not tell the user to run Grok or
Claude there. `install.sh` still installs those CLIs (same home toolchain);
leave them unless the user asks to skip.

Suggested first message:

> Read AGENTS.md and set up WSL on this Windows 11 laptop from this repo.

---

## 0. Detect where you are

```powershell
# PowerShell
if ($env:WSL_DISTRO_NAME) { "wsl:$env:WSL_DISTRO_NAME" }
elseif ($IsWindows -or $env:OS -eq 'Windows_NT') { "windows" }
else { "unknown" }
```

```bash
# bash
if [ -n "${WSL_DISTRO_NAME:-}" ]; then echo "wsl:$WSL_DISTRO_NAME"
elif [ "$(uname -s)" = Linux ]; then echo "linux-not-wsl"
else echo "unknown"; fi
```

| You are | Do this |
|---|---|
| Windows PowerShell / cmd / Git Bash | **§1**. Never run `install.sh` here. |
| WSL, distro `Ubuntu-26.04` | **§2**. |
| WSL, any other distro | Stop. This repo is **Ubuntu 26.04 only**. Do not fall back to 22.04/24.04. |
| Linux that is not WSL | Stop. This is a WSL workstation, not a bare-metal Linux install. |

`uname -m` must be `x86_64`. `install.sh` fetches amd64 tarballs (Go, gh,
saml2aws, AWS CLI, pwsh). Stop on ARM.

---

## 1. Windows-host playbook (work laptop)

Goal: Ubuntu 26.04 exists, the default user is **passwordless**, `wsl` and
`ubuntu` open that distro at `~` from Windows Terminal, this repo lives on
the **Linux** disk, and `install.sh` has been run **inside** that distro.

### 1.1 Preconditions

- Windows 11, virtualization enabled, user can elevate if `wsl --install` needs it.
- Network can reach GitHub, Microsoft packages, and the other upstreams in `install.sh`.
- First clone of **this** repo on Windows: **HTTPS**
  (`https://github.com/pjmagee/wsl-setup.git`). 1Password SSH is not ready yet.

If `wsl --install` is blocked by policy, stop and tell the user IT must allow
WSL 2. Do not try Hyper-V workarounds or a different distro.

### 1.2 One command

From the repo root in PowerShell (not Git Bash):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\windows\bootstrap.ps1
```

That script is the host orchestrator. It:

1. Upserts `guiApplications=true` in `%USERPROFILE%\.wslconfig` (does not
   clobber other keys).
2. Writes Ubuntu cloud-init
   (`%USERPROFILE%\.cloud-init\Ubuntu-26.04.user-data`) **before** first
   launch so there is **no username/password OOBE prompt**.
3. Installs WSL 2 + `Ubuntu-26.04` (`--no-launch` when the flag exists).
4. Creates or reuses the default Linux user (sanitized `$env:USERNAME`, or
   `-UserName`), **locks the password on new accounts**, and writes
   NOPASSWD sudo + `/etc/wsl.conf` `[user] default=` via
   [`windows/ensure-user.sh`](windows/ensure-user.sh) as **root**
   (`wsl -u root` — no sudo password needed).
5. `wsl --set-default Ubuntu-26.04` so a bare `wsl` hits this distro.
6. Puts `ubuntu.cmd` on the user PATH (`%USERPROFILE%\.wsl-setup\bin`).
7. Adds PowerShell functions `wsl` / `ubuntu` (bare `wsl` → `wsl.exe --cd ~`).
8. Installs Windows Terminal fragment profiles **Ubuntu** and **wsl**, both
   `wsl.exe -d Ubuntu-26.04 --cd ~`, and sets **Ubuntu** as
   `defaultProfile`.
9. Clones this repo to `~/code/wsl-setup` **inside** the distro and runs
   `install.sh`.

If WSL was just enabled, Windows may ask for a **reboot**. Re-run the same
command after login.

Optional:

```powershell
.\windows\bootstrap.ps1 -UserName magaoidh
.\windows\bootstrap.ps1 -SkipLinuxInstall    # host + WSL only
```

Do not install `Ubuntu`, `Ubuntu-22.04`, or `Ubuntu-24.04`.
Do not invent a second toolchain installer.

### 1.3 Passwordless (required)

`install.sh` uses `sudo -n` and **exits** if sudo would prompt.

New distros: cloud-init + `ensure-user.sh --create` (locked password,
NOPASSWD). Existing distros: NOPASSWD for the uid-1000 user; password is
left as-is.

Do not wait for an interactive UNIX username/password dialog. If one
appears, cloud-init was late — terminate the window and re-run
`bootstrap.ps1`.

Verify:

```powershell
wsl -d Ubuntu-26.04 -- bash -lc 'whoami; sudo -n true && echo sudo-ok'
```

### 1.4 `wsl` and `ubuntu` from Windows Terminal

After a **new** Terminal window:

| Action | Result |
|---|---|
| Open Windows Terminal | Ubuntu profile, cwd `~` (Linux home, not `/mnt/c`) |
| Tab profile **Ubuntu** or **wsl** | Same |
| `ubuntu` in PowerShell / cmd | `wsl.exe -d Ubuntu-26.04 --cd ~` |
| `wsl` in PowerShell (new session) | `wsl.exe --cd ~` (args still pass through) |
| `wsl.exe -l` / `wsl --install` | Unchanged (call `wsl.exe` or pass args) |

`wsl` from PowerShell **without** the profile function still starts on
`/mnt/c/...` (9P, slow). Always `--cd "~"` or use the Terminal profile.

### 1.5 After bootstrap

Tell the user to reopen
`\\wsl$\Ubuntu-26.04\home\<user>\code\wsl-setup` in VS Code
(**WSL: Reopen Folder in WSL**). Further edits happen there.

### 1.6 Windows leftovers (this repo does not install them)

| Host app | Why |
|---|---|
| [Docker Desktop](https://docs.docker.com/desktop/features/wsl/) + WSL integration for **Ubuntu-26.04** | `docker` / Dagger |
| [VS Code](https://code.visualstudio.com/) + [WSL extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-wsl) | `code .` from a Linux path |
| 1Password for Windows, signed in, **Use the SSH agent** on | SSH / Git |
| Windows **OpenSSH Authentication Agent** service **off** | 1Password owns `\\.\pipe\openssh-ssh-agent` |
| At least one SSH key in 1Password available to the agent | `ssh-add -l` |

SSH host config: `%USERPROFILE%\.ssh\config`, not WSL `~/.ssh/config`.
Commit signing: 1Password app → **Configure Commit Signing** → **Configure for WSL** → paste into WSL `~/.gitconfig`.

### 1.7 Verify

New Ubuntu tab (so `~/.bashrc` loads), or:

```powershell
wsl -d Ubuntu-26.04 --cd "~" -- bash -lic 'command -v git; git config --global --get core.sshCommand; type ssh; sudo -n true && echo sudo-ok'
```

Expect `core.sshCommand=ssh.exe`, `ssh` aliased to `ssh.exe`, `sudo-ok`.

---

## 2. Already-in-WSL playbook

Passwordless sudo must already work. If `sudo -n true` fails, go back to
Windows and run `windows/bootstrap.ps1` (or
`wsl -d Ubuntu-26.04 -u root -- bash windows/ensure-user.sh "$(id -un)"`
from the repo on `/mnt/c`).

```bash
. /etc/os-release
echo "$ID $VERSION_ID"    # must be ubuntu 26.04
uname -m                  # must be x86_64
sudo -n true
mkdir -p ~/code
if [ ! -d ~/code/wsl-setup/.git ]; then
  git clone https://github.com/pjmagee/wsl-setup.git ~/code/wsl-setup
fi
cd ~/code/wsl-setup
./install.sh
```

Then open a **new** Ubuntu tab. From a Linux path: `code .`

Updates later: `cd ~/code/wsl-setup && git pull && ./install.sh`

---

## 3. Invariants (do not violate)

- **Linux binaries only.** `install.sh` ignores `/mnt/c/...` and `*.exe` via
  `is_linux_bin`. Do not "fix" a missing tool by putting a Windows copy on `PATH`.
- **PATH is Linux-first.** Marked block `>>> wsl-linux-path >>>` in `~/.bashrc`.
- **Repos live on the Linux disk** (`~/code/...`), not `/mnt/c` or `/mnt/d`.
- **Do not install in WSL:** Linux VS Code, Discord, Oh My Posh. Prompt is
  Starship. Editors stay on Windows.
- **Do not reinstall `wslu` / `wslview`.** Ubuntu 26.04 dropped them. Links go
  through `scripts/wsl-open` (`BROWSER` / `GH_BROWSER`).
- **Do not configure Linux `~/.ssh/config` for 1Password.** That belongs on Windows.
- **Do not move the Linux toolchain into PowerShell.**
  [`windows/bootstrap.ps1`](windows/bootstrap.ps1) is host-only (WSL,
  cloud-init, Terminal, launchers). [`install.sh`](install.sh) is the only
  toolchain installer.
- **Do not use `apt` for language runtimes** (Node, Go, Rust, .NET, modern Python).
  Those come from upstream installers in `install.sh`. System packages only in
  [`packages/apt.txt`](packages/apt.txt).

---

## 4. 1Password SSH (already implemented)

Docs: https://www.1password.dev/ssh/integrations/wsl

`ensure_1password_ssh` writes the documented aliases to `~/.bash_aliases` and
`~/.zshrc` and sets `git config --global core.sshCommand ssh.exe`:

```bash
alias ssh='ssh.exe'
alias ssh-add='ssh-add.exe'
```

WSL forwards the **whole SSH request** to Windows `ssh.exe`, which talks to the
1Password agent. That is not `SSH_AUTH_SOCK` forwarding.

The Linux `op` CLI is unrelated to the agent (no Windows Hello).

Smoke test after §1.6: `ssh-add -l` then `ssh -T git@github.com`.

---

## 5. Changing this repo

Keep `install.sh` **idempotent** (`set -euo pipefail`). Re-runs must be safe.
Keep `windows/bootstrap.ps1` **Windows PowerShell 5.1 compatible** (work
laptops may not have PS7 yet). No `&&` / `??` / `$IsWindows` in that script.

| Change | Where |
|---|---|
| Add an apt package | [`packages/apt.txt`](packages/apt.txt) only |
| Add a toolchain | New `install_*` function, `is_linux_bin` guard, call from `main`, line in `print_summary` |
| Shell snippet | Marked block via `upsert_marked_block` (`>>> name >>>` / `<<< name <<<`) |
| Starship defaults | [`starship.toml`](starship.toml) — only copied if `~/.config/starship.toml` is absent |
| Link opener | [`scripts/wsl-open`](scripts/wsl-open) |
| WSL / Terminal / passwordless user | [`windows/bootstrap.ps1`](windows/bootstrap.ps1), [`windows/ensure-user.sh`](windows/ensure-user.sh) |

Comments: short, factual, only for non-obvious constraints.
Do not leave placeholders for unrelated work.

---

## 6. File map

```
install.sh                 # Linux toolchain; run only inside Ubuntu 26.04
packages/apt.txt           # apt packages (no language runtimes)
scripts/wsl-open           # http(s)/mailto → Windows default browser
starship.toml              # seed config (scan_timeout for /mnt/c)
windows/bootstrap.ps1      # host orchestrator (run from Windows)
windows/ensure-user.sh     # root: user + NOPASSWD + /etc/wsl.conf
windows/ubuntu.cmd         # ubuntu → wsl.exe -d Ubuntu-26.04 --cd ~
README.md                  # human docs
AGENTS.md                  # this playbook
.github/copilot-instructions.md
```

Definition of done for a work-laptop bootstrap:

1. `wsl -d Ubuntu-26.04 -- echo ok` works; `wsl -l` default is `Ubuntu-26.04`.
2. `sudo -n true` works inside the distro. New users have no login password.
3. Windows Terminal default profile is **Ubuntu** at `~`. Profiles **Ubuntu**
   and **wsl** exist. `ubuntu` launches the same session.
4. `~/code/wsl-setup` is a git checkout on the Linux disk.
5. `./install.sh` finished; summary shows Linux versions and `sudo passwordless`.
6. `1p-ssh` aliases → `ssh.exe`; `git-ssh` is `ssh.exe`.
7. User knows the Windows leftovers (Docker, VS Code WSL, 1Password agent).
