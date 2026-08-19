---
title: New WSL vs restore
description: When to remount an existing distro, and when to create Ubuntu 26.04 from scratch.
order: 5
group: Use
---

## Restore an existing distro

If Collect ticked your WSL disk, Restore imports it **in place**. Same username, same Homebrew prefix, same `~/code`. Nobody runs `install.sh`.

## New WSL

Use this when there is **no disk to bring back** — a work laptop, or a home PC after a reset with no kit.

1. **New WSL** → pick a linux profile (`home`, `work`, or a custom id).
2. The exe enables WSL if needed, installs **Ubuntu-26.04** only (`--no-launch`), waits for first boot, creates uid 1000 from your Windows username (empty password + NOPASSWD sudo), sets the default distro, then runs `install.sh <profile>` inside Ubuntu.

You never clone. The exe clones the installer **inside** WSL.

If Windows asks for a reboot after enabling WSL, reboot and run New WSL again.

## Apply a bundle

**Profiles → Apply** (or `windows-wsl-setup apply home`) can winget-install the Windows half and then do the New WSL path. Still not a kit restore: no Dev Drive, no VHDX, no bookmarks.
