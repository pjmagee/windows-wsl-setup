---
title: Getting started
description: Download the exe, pick a path, and finish what the binary cannot click.
order: 1
group: Start
---

**Windows WSL Manager** is one Windows console app. You do not clone this repo to use it.

Four modes, same binary:

| You have | You want | Run |
|---|---|---|
| A used Windows 11 PC | A snapshot that survives a reinstall | **Collect** |
| That snapshot on a data drive | The same apps and disks on a fresh Windows 11 | **Restore** |
| No snapshot | A new Linux environment | **New WSL** |
| A named software list | Winget packages and/or Linux CLIs | **Profiles** → **Apply** |

A **kit** is a snapshot of *this* machine. A **profile** is a list of *what you want*. Restore wins when the Linux disk still exists. Apply does not remount disks.

## 1. Get the exe

1. Open [Releases](https://github.com/pjmagee/windows-wsl-manager/releases).
2. Download `windows-wsl-setup.exe` (current release name of the manager).
3. Run it. The home screen is Collect / Restore / New WSL / Profiles.

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
```

No installer wizard. If Windows just enabled WSL, it may ask to reboot — reboot, then run the exe again.

Prefer an agent? [Install the skill](../agents/), then ask it to download the latest release and drive Collect, Restore, New WSL, or Apply.

## 2. Pick the path

**Collect** (used PC). Destination must **not** be `C:` and should not be a volume you might format later. Tick winget apps by category, Linux disks to keep, and host leftovers (Dev Drive, Docker data, Brave bookmarks, dotfiles). The kit folder gets `KIT.json`, a copy of the exe, and `START-HERE.txt`.

**Restore** (fresh Windows 11 + a kit). Leave data drives alone. The exe looks for `KIT.json` under `D:`–`Z:\Backups`. Tick packages, confirm remounting disks, then Apply. Brave bookmarks are copied; `extensions.html` opens so you click **Add** yourself.

**New WSL** (no disk to bring back). Pick a distro still listed in `wsl --list --online`: **Ubuntu-26.04** (default), **Debian**, or **archlinux**. Then a linux profile (`home`, `work`, or custom). The exe enables WSL if needed, installs that distro, creates a Linux user from your Windows username, turns on passwordless sudo, clones the Linux installer inside the distro, and runs it.

**Profiles** (no kit). Tick Windows and Linux packages, optionally create WSL, then Apply. `g` drafts a profile from what is already installed. `s` saves a user profile. Apply **installs**; it never uninstalls Windows apps. Linux setup may drop CLIs that are not on the chosen linux profile.

Details: [Kits](../kits/), [Profiles](../profiles/), [New Linux vs restore](../wsl/).

## 3. What the exe cannot click

Do these once, after Restore or New WSL:

- **1Password** — sign in, enable **Use the SSH agent**. Turn the Windows OpenSSH Authentication Agent service **off**.
- **Brave** — click **Add** on each store page from `extensions.html`.
- **Steam / Epic** — add the existing library folder; do not re-download games that already sit on a data drive.
- **Docker Desktop** — enable WSL integration for the distro you restored or created (**Ubuntu-26.04** if that is the new default).
- **VS Code** — install the WSL extension; open a Linux path, not `/mnt/c`.
- Linux disk **Access Denied** after remount: grant SID `S-1-5-83-0` Full control on that VHDX.

Never format a data drive to “clean up.” Never `wsl --unregister` unless you meant to delete that disk.
