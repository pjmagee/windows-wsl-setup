---
name: windows-wsl-setup
description: >
  Collect or restore a Windows 11 PC, create a new WSL distro (Ubuntu, Debian, or Arch),
  or apply a software profile. Use when the user has a messy PC, is about to reset
  Windows, just reinstalled, wants an agent to download the CLI and run collect/restore,
  or mentions kits, profiles, WSL disks, or data volumes.
---

# Windows WSL Setup — agent skill

The product is **`windows-wsl-setup.exe`** from GitHub Releases.

Humans and agents **do not clone** this repo to use it. Clone only if you are changing the product.

Install this skill on other agents:

```
npx skills add pjmagee/windows-wsl-setup
```

## Install the CLI

On Windows, download the latest exe (do not build unless developing):

```
gh release download -R pjmagee/windows-wsl-setup -p windows-wsl-setup.exe --dir $env:USERPROFILE\Downloads
```

Or open https://github.com/pjmagee/windows-wsl-setup/releases and take `windows-wsl-setup.exe`.

Then run it. JSON commands print JSON on stdout; errors on stderr.

## Modes

| Mode | When |
|---|---|
| **Collect** | Used PC. Snapshot apps, Linux disks, data volumes to a non-system drive. |
| **Restore** | Fresh Windows 11 + a kit. Install apps, remount disks, restore browser bookmarks. |
| **New WSL** | No kit. Create **Ubuntu-26.04** (default), **Debian**, or **archlinux**, then a linux profile. |
| **Profiles / Apply** | Named software lists. Shipped `default` / `home` / `work`. |

A **kit** is what this PC has. A **profile** is what they want. Restore wins when the Linux disk still exists. Apply does not remount disks.

Never format data drives. Never `wsl --unregister` unless they confirm.

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

Homebrew installs the CLIs on all three. `apt` or `pacman` is only the bootstrap set.

Do not offer Fedora, Kali, or openSUSE as a new-instance target. Restoring a kit that already has them is fine.

## CLI

```
windows-wsl-setup suggest
windows-wsl-setup map <winget-id>
windows-wsl-setup search winget <query>
windows-wsl-setup search linux <query>
windows-wsl-setup catalog linux|windows
windows-wsl-setup profile list|show|new|add|remove
windows-wsl-setup apply <id>
windows-wsl-setup apply <id> --windows-only|--linux-only
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
```

Prefer Linux for SDKs and cloud CLIs (`map` / `preferLinux`). Games, browsers, editors, password managers, and Docker Desktop stay on Windows.

## Leftovers the exe cannot click

- Password manager → enable the SSH agent
- Game launcher → add the existing library folder
- Docker → Linux integration for the restored distro
- Browser → **Add** on the extensions page
- Linux disk Access Denied after remount: grant SID `S-1-5-83-0` Full control
