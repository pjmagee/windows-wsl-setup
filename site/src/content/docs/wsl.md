---
title: New Linux vs restore
description: When to remount an existing distro, and when to create a new one.
order: 6
group: Use
---

## Restore an existing distro

If Collect ticked your Linux disk, Restore imports that VHDX in place. Same user, same files, same Homebrew prefix. Nobody reinstalls packages. Other distros already on the machine stay listed.

## New WSL

Use this when there is **no disk to bring back**.

1. **New WSL** → pick a distro still in `wsl --list --online`: **Ubuntu-26.04** (default), **Debian**, or **archlinux**. Then a linux profile (`home`, `work`, or a custom id).
2. The exe enables WSL if needed (reboot and re-run if Windows asks), installs that distro, and:
   - creates uid 1000 from your Windows username when the distro has no user yet
   - turns on passwordless sudo
   - sets that distro as the WSL default
   - clones `pjmagee/windows-wsl-manager` to `~/code/windows-wsl-manager` **inside Linux**
   - runs `install.sh` for the profile you picked (apt or pacman bootstrap, then Homebrew CLIs)

You never clone on Windows. Fedora, Kali, and openSUSE are not new-instance targets; restoring a kit that already has them is fine.

`uname -m` must be `x86_64`. Homebrew bottles and Compass are amd64.

## Apply a bundle

**Profiles → Apply** can install Windows apps (winget, never uninstall) and then take the New WSL path. Still not a kit restore: no disk remount, no Brave bookmarks.

Linux `install.sh work` does **not** install grok / claude / opencode / hugo / stripe; if those binaries are already present it removes them. `home` keeps the home agents.

Official targets and package managers: [Distros](../distros/).
