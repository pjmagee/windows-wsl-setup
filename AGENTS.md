# AGENTS.md

Playbook for **Linux toolchain / work-laptop** agents.

Home PC reset (**Collect** / **Restore**) and empty-Ubuntu (**New WSL**) are
**not this file**. That is `wwm.exe`. See [README.md](README.md)
and `.agents/skills/wwm-cli/SKILL.md`.

You are bootstrapping the **same WSL 2 Ubuntu 26.04 workstation** this repo
defines — typically a **clean Windows 11 work laptop** that has none of the
home toolchain. Clone this repo, then **execute** the playbook. Do not invent
a second Linux installer.

At work, **Copilot is the operator**. Do not tell the user to run Grok or
Claude there. Run the **work** profile (`linux/install.sh work` /
`windows/bootstrap.ps1`). That profile installs Copilot CLI and **does not**
install grok, claude, opencode, devtunnel, changie, hugo, or stripe. If those
binaries are already present, `install.sh work` removes them.

At home, run `./linux/install.sh home`.

Suggested first message (work laptop / first WSL):

> Read AGENTS.md and set up WSL on this Windows 11 laptop from this repo.

Suggested first message (used PC, about to reset, or fresh PC with a kit):

> Run wwm. Collect or Restore. Finish anything the exe cannot click (1Password SSH, Steam library, Docker WSL, Brave Add buttons). Do not clone. Do not run install.sh — the restored WSL disk already has Linux tools.

Suggested first message (empty Ubuntu / no kit):

> Run wwm → New WSL → home (or work). Always Ubuntu.

The exe is Collect / Restore / New WSL. Copilot on a work laptop must **not** run Collect/Restore unless the human asked. Skill: `.agents/skills/wwm-cli/SKILL.md`.

---

## 0. Detect where you are

```powershell
# PowerShell
if ($env:WSL_DISTRO_NAME) { "wsl:$env:WSL_DISTRO_NAME" }
elseif ($IsWindows -or $env:OS -eq 'Windows_NT') { "windows" }
else { "unknown" }
```

```bash
# bash (Git Bash is not Linux — uname is MINGW*/MSYS* and this prints "windows")
if [ -n "${WSL_DISTRO_NAME:-}" ]; then echo "wsl:$WSL_DISTRO_NAME"
elif [ "$(uname -s)" = Linux ]; then echo "linux-not-wsl"
else echo "windows"; fi
```

| You are | Do this |
|---|---|
| Windows, human asked to **capture / backup** a used PC | **§0a**. `wwm.exe` **Collect**. Do not run `install.sh`. |
| Windows, human pointed at a **kit** (`KIT.json` on a data drive) | **§0a**. `wwm.exe` **Restore**. Do not format data drives. Do not `wsl --unregister`. |
| Windows, human wants a **new empty Linux** (no kit) | **§0a**. `wwm.exe` **New WSL**. Work laptop: Ubuntu-26.04. Other create ids: [site distros](site/src/content/docs/distros.md). |
| Windows, human wants a **default / custom software set** (no kit) | **§0a**. `wwm.exe` **Profiles** or `apply <id>`. |
| Windows PowerShell / cmd / Git Bash (cloned repo, work laptop) | **§1**. Never run `install.sh` here. Git Bash: invoke `powershell.exe`, do not treat this as Linux. |
| WSL, distro `Ubuntu-26.04` | **§2**. |
| WSL, any other distro (`Ubuntu`, `Ubuntu-24.04`, …) | Wrong distro. Do **not** run `install.sh` here. Do **not** `wsl --unregister`. Run `windows\bootstrap.ps1` from **Windows PowerShell** (from this shell: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File <windows-clone>/windows/bootstrap.ps1` if the repo is on `/mnt/c` or `/mnt/d`). That adds `Ubuntu-26.04` beside this distro. Then continue in 26.04. |
| Linux that is not WSL | Stop. This is a WSL workstation, not a bare-metal Linux install. |

### 0a. Capture a used machine (Windows WSL Manager)

End users download `wwm.exe` from GitHub Releases (no clone).

```
wwm              Collect / Restore / New WSL
wwm collect      scan this PC, write a kit (winget manifest, WSL, Dev Drive, Brave)
wwm restore      tick packages from the kit → winget install; remount Dev Drive / WSL; bookmarks + extensions.html
wwm new-wsl      pick Ubuntu / Debian / archlinux + a linux profile
wwm profiles     edit / suggest / apply a named software profile
wwm suggest      draft Windows+Linux profile from winget list (JSON)
wwm apply home   winget + optional New WSL (not a kit restore)
```

Do not tell the user to run PowerShell scripts or git clone for this. Maintainers build `windows/cli`.

Never write the kit on `C:`. Never unregister WSL. Restore brings the **existing** Linux disk back; do not run New WSL unless they have no disk to restore.

New WSL asks which **supported** distro Microsoft still lists (`wsl --list --online`): latest per family (Ubuntu-26.04 default). Pengwin is not a create target. apt, pacman, dnf, or zypper bootstrap, then Homebrew.

`uname -m` must be `x86_64`. Homebrew bottles and Compass are amd64.
Stop on ARM.

---

## 1. Windows-host playbook (work laptop)

Humans without a clone: `wwm.exe` → **New WSL** → **work**.
This section is for Copilot (or you) **after this repo is already cloned**.

Goal: Ubuntu 26.04 exists on this **already-present** work laptop, `sudo -n`
works for the user that is already on that distro, `wsl` and `ubuntu` open
it at `~` from Windows Terminal, this repo lives on the **Linux** disk, and
`install.sh` has been run **inside** that distro.

### 1.1 Preconditions

- Windows 11, virtualization enabled, user can elevate if `wsl --install` needs it.
- Network can reach GitHub, Microsoft packages, and the other upstreams in `install.sh`.
- First clone of **this** repo on Windows: **HTTPS**
  (`https://github.com/pjmagee/wwm.git`). 1Password SSH is not ready yet.

If `wsl --install` is blocked by policy, stop and tell the user IT must allow
WSL 2. Do not try Hyper-V workarounds or a different distro.

### 1.2 One command

From the repo root in PowerShell (not Git Bash):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\windows\bootstrap.ps1
```

That script is the host orchestrator for a laptop that **already exists**.
It does **not** use cloud-init and does **not** create or lock a Linux user.

1. Upserts `guiApplications=true` in `%USERPROFILE%\.wslconfig` (does not
   clobber other keys).
2. `wsl --update` (best-effort) and prints `wsl -l -v`. Ubuntu 26.04's
   `.wsl` image needs WSL **2.4.10+**. Does not unregister anything.
3. Installs `Ubuntu-26.04` only if it is missing (`wsl --install`).
   If Ubuntu is brand new, the user completes the normal username/password
   prompt once ([Microsoft](https://learn.microsoft.com/en-us/windows/wsl/setup/environment#set-up-your-linux-username-and-password));
   after that WSL auto-signs-in. Then re-run this script.
4. NOPASSWD sudo + `/etc/wsl.conf` `[user] default=` for the **existing**
   uid-1000 user, via [`linux/ensure-user.sh`](linux/ensure-user.sh)
   as root (`wsl -u root`).
5. `wsl --set-default Ubuntu-26.04` so a bare `wsl` hits this distro.
6. Puts `ubuntu.cmd` on the user PATH (`%USERPROFILE%\.wwm`).
7. Adds PowerShell functions `wsl` / `ubuntu` (bare `wsl` → `wsl.exe ~`).
8. Windows Terminal keeps the official **Microsoft.WSL** tabs. Do not
   overwrite their `commandline` (that passed `/home/<user>` as a command).
   WWM only disables the old penguin generator and adds a **wsl** launcher.
   Work laptop still sets Ubuntu as `defaultProfile`. Sidecar distros do not
   steal the default.
9. Clones this repo to `~/code/wwm` **inside** the distro and runs
   `linux/install.sh work`. If a required host step fails, the host script
   **throws** (no fake `Done.`).

If WSL was just enabled, Windows may ask for a **reboot**. Re-run the same
command after login.

```powershell
.\windows\bootstrap.ps1 -SkipLinuxInstall    # host + WSL only
```

Do not install `Ubuntu`, `Ubuntu-22.04`, or `Ubuntu-24.04`.
Do not `wsl --unregister` leftover distros unless the user asks.
Do not invent a second toolchain installer.
Do not write `%USERPROFILE%\.cloud-init\`.

### 1.2a Leftover distros (24.04 / Store Ubuntu)

A used work laptop often already has `Ubuntu-24.04`, Store `Ubuntu`,
and/or `docker-desktop`. That is fine. This repo adds **Ubuntu-26.04**
next to them.

```powershell
wsl.exe -l -v
```

| Rule | Why |
|---|---|
| Leave other distros installed | Unregistering deletes their disk. Unused 24.04 can sit there. |
| Never `wsl --unregister` unless the user asked | Copilot must not "clean up" the old Ubuntu. |
| Default becomes `Ubuntu-26.04` | Bare `wsl` / new Terminal **Ubuntu** tab hit 26.04. Old Store/Terminal profiles may still launch 24.04. |
| `ubuntu` in **cmd** may still be Store `ubuntu.exe` | `PATHEXT` tries `.EXE` before `.CMD`, so `%USERPROFILE%\.wwm\ubuntu.cmd` loses to `ubuntu.exe`. PowerShell's `ubuntu` function and the fragment profiles are the 26.04 launchers. |
| Docker Desktop WSL integration | Often still attached to 24.04. User must enable **Ubuntu-26.04** in Docker Desktop (leftover in §1.6). |

Tell the user the old distro is still listed. Do not migrate files out of it
unless they ask.

### 1.3 Passwordless sudo (required for install.sh)

`install.sh` uses `sudo -n` and **exits** if sudo would prompt.

WSL already auto-signs-in after first-run; that is not the same as
NOPASSWD sudo. `ensure-user.sh` only adds sudoers for the existing user.
It does not lock the password.

Verify:

```powershell
wsl -d Ubuntu-26.04 -- bash -lc 'whoami; sudo -n true && echo sudo-ok'
```

### 1.4 `wsl` and `ubuntu` from Windows Terminal

After a **new** Terminal window:

| Action | Result |
|---|---|
| Open Windows Terminal | Ubuntu profile, cwd `~` (Linux home, not `/mnt/c`) |
| Tab profile **Ubuntu-26.04** / **Debian** / … | Official Microsoft.WSL tabs. WWM does not replace them. |
| Tab profile **wsl** | WSL default distro at `~` |
| `ubuntu` in PowerShell (new session) | `wsl.exe -d Ubuntu-26.04 ~` |
| `ubuntu` in cmd | 26.04 via `ubuntu.cmd`, unless Store `ubuntu.exe` wins (`PATHEXT`) |
| `wsl` in PowerShell (new session) | `wsl.exe ~` (args still pass through) |
| `wsl.exe -l` / `wsl --install` | Unchanged (call `wsl.exe` or pass args) |

`wsl` from PowerShell **without** the profile function still starts on
`/mnt/c/...` (9P, slow). Always `wsl ~` or use the Terminal profile.

### 1.5 After bootstrap

Tell the user to reopen
`\\wsl$\Ubuntu-26.04\home\<user>\code\wwm` in VS Code
(**WSL: Reopen Folder in WSL**). Further edits happen there.

### 1.6 Windows leftovers (this repo does not install them)

| Host app | Why |
|---|---|
| [Docker Desktop](https://docs.docker.com/desktop/features/wsl/) + WSL integration for **Ubuntu-26.04** | `docker` / Dagger. If 24.04 was the old default, flip the integration checkbox — this repo does not do that. |
| [VS Code](https://code.visualstudio.com/) + [WSL extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-wsl) | `code .` from a Linux path |
| 1Password for Windows, signed in, **Use the SSH agent** on | SSH / Git |
| Windows **OpenSSH Authentication Agent** service **off** | 1Password owns `\\.\pipe\openssh-ssh-agent` |
| At least one SSH key in 1Password available to the agent | `ssh-add -l` |

SSH host config: `%USERPROFILE%\.ssh\config`, not WSL `~/.ssh/config`.
Commit signing: 1Password app → **Configure Commit Signing** → **Configure for WSL** → paste into WSL `~/.gitconfig`.

### 1.7 Verify

New Ubuntu tab (so `~/.bashrc` loads), or:

```powershell
wsl -d Ubuntu-26.04 ~ -- bash -lic 'command -v git; git config --global --get core.sshCommand; type ssh; sudo -n true && echo sudo-ok'
```

Expect `core.sshCommand=ssh.exe`, `ssh` aliased to `ssh.exe`, `sudo-ok`.

---

## 2. Already-in-WSL playbook

Passwordless sudo must already work. If `sudo -n true` fails, go back to
Windows and run `windows/bootstrap.ps1` (or
`wsl -d Ubuntu-26.04 -u root -- bash linux/ensure-user.sh "$(id -un)"`
from the repo on `/mnt/c`).

```bash
. /etc/os-release
echo "$ID $VERSION_ID"    # must be ubuntu 26.04
uname -m                  # must be x86_64
sudo -n true
mkdir -p ~/code
if [ ! -d ~/code/wwm/.git ]; then
  git clone https://github.com/pjmagee/wwm.git ~/code/wwm
fi
cd ~/code/wwm
# Work laptop (Copilot):
./linux/install.sh work
# Home machine (Grok / Claude):
./linux/install.sh home
```

Then open a **new** Ubuntu tab. From a Linux path: `code .`

The chosen profile is saved in `~/.wwm/profile`. Later updates:

```bash
cd ~/code/wwm && git pull && ./linux/install.sh
brew update && brew upgrade    # already-installed Homebrew CLIs
```

Bare `./linux/install.sh` reuses the saved profile, or **home** if none is saved.

Profiles:

| Profile | Steps | Homebrew |
|---|---|---|
| `blank` | distro + Linux user + passwordless sudo. No `linux/install.sh` toolchain. | none |
| `home` | [`linux/base.txt`](linux/base.txt) | IDs in [`linux/profiles/home.json`](linux/profiles/home.json) |
| `work` | same `base.txt` steps | [`linux/profiles/work.json`](linux/profiles/work.json). Prunes IDs not on that list. |
| custom | `base.txt`, plus `linux/<id>.txt` if you add extra `install_*` | `~/.wwm/profiles/<name>.json` |

Catalog (categories, no home/work flags): [`linux/catalog.json`](linux/catalog.json). Overlay `{ "tools": ["id", ...] }` at `~/.wwm/linux-profile.json` replaces the ID list.

---

## 3. Invariants (do not violate)

- **Layout.** `linux/` is the in-distro toolchain (`install.sh`, packages,
  Starship, wsl-open). `windows/` is the host product (`wwm.exe`, winget
  catalogs, `install.ps1`, bootstrap). Do not put Linux installers or
  Starship config at the repo root.
- **No migration. No compatibility shims. No tech debt.** This product has
  no installed user base. Do not copy from old paths, accept old names, or
  keep both the old and new thing. If you rename a file, dir, env var, profile
  id, or git remote, **delete the old name**. Do not add README-only
  directories, unused files, or comments that exist only to warn about a path
  that is gone.
- **One state dir: `.wwm`.** Windows `%USERPROFILE%\.wwm` (`wwm.exe`,
  `ubuntu.cmd`, user profiles). Linux `~/.wwm` (saved profile, overlay,
  custom linux profiles). Repo on the Linux disk: `~/code/wwm`. Env override
  for the repo root: `WWM_ROOT` only.
- **Linux binaries only.** `install.sh` ignores `/mnt/c/...` and `*.exe` via
  `is_linux_bin`. Do not "fix" a missing tool by putting a Windows copy on `PATH`.
- **PATH is Linux-first.** Marked block `>>> wsl-linux-path >>>` in `~/.bashrc`.
- **Repos live on the Linux disk** (`~/code/...`), not `/mnt/c` or `/mnt/d`.
- **Do not install in WSL:** Linux VS Code, Discord, Oh My Posh. Prompt is
  Starship. Editors stay on Windows.
- **apt for system packages. Homebrew for CLIs and language runtimes.**
  Official Linux/WSL prefix only: `/home/linuxbrew/.linuxbrew` (needed for
  bottles). `brew update && brew upgrade` is the “update all CLIs” command.
  Do not add a third package manager. Compass (Linux GUI) and Cloudflare `cf`
  are not in Homebrew; those stay as special `install_*` steps.
- **Shipped linux profiles are `home` and `work`.** They are ID lists in
  `linux/profiles/`. Custom names are allowed (file must exist). Categories
  live on the catalog, not on the profile.
- **Do not reinstall `wslu` / `wslview`.** Ubuntu 26.04 dropped them. Links go
  through `linux/scripts/wsl-open` (`BROWSER` / `GH_BROWSER`).
- **Do not configure Linux `~/.ssh/config` for 1Password.** That belongs on Windows.
- **Do not move the Linux toolchain into PowerShell.**
  [`windows/bootstrap.ps1`](windows/bootstrap.ps1) is host-only (WSL,
  Terminal, launchers, NOPASSWD for the existing user).
  [`linux/install.sh`](linux/install.sh) is the only toolchain installer (it installs
  Homebrew and runs the Brewfiles).
- **Do not add cloud-init.** The work laptop already exists; first-run
  user creation is the normal WSL prompt if Ubuntu is new.
- **Do not unregister leftover distros.** Unused `Ubuntu-24.04` / Store
  `Ubuntu` stay installed unless the user asks to remove them.
- **Do not use `apt` for language runtimes** (Node, Go, Rust, .NET, modern Python).
  Those come from Homebrew, then uv / fnm / rustup for the versioned
  runtimes. System packages only in [`linux/packages/apt.txt`](linux/packages/apt.txt)
  (or dnf/pacman/zypper next to it).

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

Keep `linux/install.sh` **idempotent** (`set -euo pipefail`). Re-runs must be safe.
Optional `install_*` steps go through `run_step` so a blocked GitHub / vendor
host cannot abort the rest of the run. Required host steps (bashrc, sudo,
`wsl-open`) still fail the script.
Keep `windows/bootstrap.ps1` **Windows PowerShell 5.1 compatible** (work
laptops may not have PS7 yet). No `&&` / `??` / `$IsWindows` in that script.

Do not add migrate_*, copy-from-old-dir, “if the old name exists”, or env
aliases for renamed variables. Idempotent re-runs of **current** files are
fine (upsert marked blocks, brew upgrade). Cleaning a previous product name
is not.

| Change | Where |
|---|---|
| Add an apt package | [`linux/packages/apt.txt`](linux/packages/apt.txt) only |
| Add a dnf / zypper package | [`linux/packages/dnf.txt`](linux/packages/dnf.txt) / [`linux/packages/zypper.txt`](linux/packages/zypper.txt) |
| Add a CLI / runtime | Entry in [`linux/catalog.json`](linux/catalog.json) + id on the right `linux/profiles/<name>.json`, line in `print_summary` |
| Post-brew step (uv/fnm/rustup, Compass, `cf`) | New `install_*` function, line in [`linux/base.txt`](linux/base.txt) (or `linux/<id>.txt` for one profile) |
| Shell snippet | Marked block via `upsert_marked_block` (`>>> name >>>` / `<<< name <<<`) |
| Starship defaults | [`linux/starship.toml`](linux/starship.toml) — only copied if `~/.config/starship.toml` is absent |
| Link opener | [`linux/scripts/wsl-open`](linux/scripts/wsl-open) (`open` symlink). Clipboard: [`linux/scripts/pbcopy`](linux/scripts/pbcopy) / [`linux/scripts/pbpaste`](linux/scripts/pbpaste) |
| WSL / Terminal / passwordless user | [`windows/bootstrap.ps1`](windows/bootstrap.ps1), [`linux/ensure-user.sh`](linux/ensure-user.sh) |
| Windows exe installer | [`windows/install.ps1`](windows/install.ps1) (`irm https://pjmagee.github.io/wwm/install.txt \| iex`) |

Do not `brew install grok` (unrelated regex tool). xAI Grok Build is `cask "grok-build"`. Flux CD is `fluxcd`, not `flux`.
Do not `brew install astro` (Astronomer). withastro is catalog kind `npm`, package `astro`.

Comments: short, factual, only for non-obvious constraints.
Do not leave placeholders for unrelated work.
Do not leave empty folders or files whose only content is “do not put X here”.

---

## 6. File map

```
linux/install.sh           # Linux toolchain (run inside the distro)
linux/ensure-user.sh       # root: NOPASSWD + /etc/wsl.conf
linux/starship.toml        # seed config (scan_timeout for /mnt/c)
linux/catalog.json         # Linux catalog (category + brew/cask)
linux/profiles/*.json      # linux profile ID lists (home, work, …)
linux/base.txt             # shared install_* steps
linux/packages/            # apt / dnf / pacman / zypper bootstrap
linux/scripts/             # linux-tools.py, wsl-open, pbcopy, pbpaste
windows/install.ps1        # irm github.io/wwm/install.txt | iex
windows/bootstrap.ps1      # host orchestrator (run from Windows)
windows/ubuntu.cmd         # ubuntu → wsl.exe -d Ubuntu-26.04 ~
windows/catalog.json       # curated winget catalog
windows/profiles/*.json    # windows profile ID lists
windows/bundles/*.json     # windows + linux + distro
windows/cli/               # wwm.exe (Collect / Restore / New WSL / Profiles)
schema/kit.schema.json     # KIT.json
schema/wwm.opencli.json    # OpenCLI description (`wwm spec`)
site/                      # GitHub Pages
skills/                    # wwm-cli skill
README.md
AGENTS.md
```

Definition of done for a work-laptop bootstrap:

1. `wsl -d Ubuntu-26.04 -- echo ok` works; `wsl -l` default is `Ubuntu-26.04`.
   Older distros may still be listed; that is OK.
2. `sudo -n true` works inside the distro for the existing Linux user.
3. Windows Terminal default profile is **Ubuntu** at `~` on a work laptop.
   Official Microsoft.WSL distro tabs remain. There is a **wsl** launcher.
   PowerShell `ubuntu` launches Ubuntu-26.04.
4. `~/code/wwm` is a git checkout on the Linux disk.
5. `./linux/install.sh work` finished; summary shows `profile work`, Homebrew,
   Copilot CLI, Linux versions, and `sudo passwordless`. No grok/claude/
   opencode/devtunnel/changie/hugo/stripe. Host bootstrap does not print
   `Done.` if `linux/install.sh` failed a required host step.
6. `1p-ssh` aliases → `ssh.exe`; `git-ssh` is `ssh.exe`.
7. User knows the Windows leftovers (Docker integration for **26.04**, VS Code
   WSL, 1Password agent) and that unused 24.04 was left installed.
