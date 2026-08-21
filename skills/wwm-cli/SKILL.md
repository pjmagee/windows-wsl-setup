---
name: wwm-cli
description: >
  Collect or restore a Windows 11 PC, create a new WSL distro (Ubuntu, Debian, or Arch),
  or apply a software profile. Use when the user has a messy PC, is about to reset
  Windows, just reinstalled, wants an agent to download the CLI and run collect/restore,
  or mentions kits, profiles, WSL disks, or data volumes.
---

# Windows WSL Manager — agent skill

The product is **`wwm.exe`** from GitHub Releases.

Humans and agents **do not clone** this repo to use it. Clone only if you are changing the product.

Install this skill at **user** scope (do not clone the repo). Pick **one**:

```
gh skill install pjmagee/wwm wwm-cli --scope user --agent grok
```

```
npx skills add pjmagee/wwm --skill wwm-cli -g -y
```

## Install the CLI

Confirm the host is Windows (not WSL, not Git Bash). Install the latest exe into `~\.wwm` — do not clone and do not build unless developing:

```
New-Item $HOME\.wwm -ItemType Directory -Force | Out-Null
Invoke-WebRequest -UseBasicParsing https://github.com/pjmagee/wwm/releases/latest/download/wwm.exe -OutFile $HOME\.wwm\wwm.exe
$env:Path = "$HOME\.wwm;$env:Path"
wwm
```

If that 404s, stop. Do not clone as a substitute.

JSON commands print JSON on stdout; errors on stderr.

Machine-readable CLI map (OpenCLI, https://opencli.org/):

```
wwm spec
```

## Modes

| Mode | When |
|---|---|
| **Collect** | Used PC. Snapshot apps, Linux disks, data volumes to a non-system drive. |
| **Restore** | Fresh Windows 11 + a kit. Install apps, remount disks, restore browser bookmarks. |
| **New WSL** | No kit. User picks a **supported** distro still in `wsl --list --online` (Ubuntu-26.04, Debian, archlinux), then a linux profile (`blank`, `home`, or `work`). |
| **Profiles / Apply** | Named software lists. Shipped `blank` / `default` / `home` / `work`. |

A **kit** is what this PC has. A **profile** is what they want. Restore wins when the Linux disk still exists. Apply does not remount disks.

Never format data drives. Never `wsl --unregister` unless they confirm (`wwm distro remove <name> --yes`).

## Linux profiles

| Id | What you get |
|---|---|
| `blank` | Distro + Linux user + passwordless sudo. No Homebrew, no `install.sh` toolchain. |
| `home` | blank host steps, then Homebrew CLIs (grok, claude, opencode, hugo, stripe, …) |
| `work` | blank host steps, then work CLIs (Copilot CLI; no grok/claude/opencode) |

```
wwm new-wsl --profile blank --distro Debian
wwm new-wsl --profile home --distro Ubuntu-26.04
```

Adding a distro beside an existing one does not steal the WSL default.

## Install order (Windows)

`apply` already sorts by catalog `priority`. If you install one-by-one yourself, use the same order:

1. Password manager (`AgileBits.1Password`)
2. Their browser (whatever they ticked — not “install every browser”)
3. Daily desktop (7-Zip, Terminal, PowerToys)
4. Editors, Git, Docker, agents
5. Games / media / cleaners last

Then ask them to unlock the password manager, open the browser, and click **Add** on the extensions page.

## Linux

Official new-instance targets: **Ubuntu-26.04**, **Debian**. **archlinux** is offered; amd64 only; first boot may be root until we create a user.

Homebrew installs the CLIs on `home` and `work`. `apt` or `pacman` is only the bootstrap set. `blank` skips that.

Do not offer Fedora, Kali, or openSUSE as a new-instance target. Restoring a kit that already has them is fine.

## CLI

```
wwm spec
wwm suggest
wwm map <winget-id>
wwm search winget <query>
wwm search linux <query>
wwm catalog linux|windows
wwm profile list|show|new|add|remove|delete
wwm profile new <id> [--from home] [--name "Media PC"]
wwm profile delete <id>
wwm apply <id>
wwm apply blank --linux-only --distro Debian
wwm apply <id> --windows-only|--linux-only --distro Debian
wwm distros
wwm distro sync
wwm distro remove <name> --yes
wwm collect
wwm restore
wwm new-wsl --profile blank --distro Debian
wwm new-wsl --profile home --distro Ubuntu-26.04
```

Windows Terminal is the host console. Official **Microsoft.WSL** tabs stay; do **not** overwrite their command line (that is what produced `/home/<user>: Is a directory`). WWM only adds a **wsl** launcher.

Prefer Linux for SDKs and cloud CLIs (`map` / `preferLinux`). Games, browsers, editors, password managers, and Docker Desktop stay on Windows.

## Leftovers the exe cannot click

- Password manager → enable the SSH agent
- Game launcher → add the existing library folder
- Docker → Linux integration for the restored distro
- Browser → **Add** on the extensions page
- Linux disk Access Denied after remount: grant SID `S-1-5-83-0` Full control
