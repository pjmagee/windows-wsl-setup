---
title: New Linux vs restore
description: When to remount an existing distro, and when to create a new one.
order: 6
group: Use
---

## Restore an existing distro

If Collect ticked your Linux disk, Restore imports it in place. Same user, same files, same toolchain. Nobody reinstalls packages.

## New WSL

Use this when there is **no disk to bring back**.

1. **New WSL** → pick a distro (Ubuntu, Debian, or Arch) and a linux profile (`home`, `work`, or custom).
2. The exe enables WSL if needed, installs that distro, creates a user from your Windows username, then bootstraps the profile (Homebrew CLIs).

You never clone. The exe clones the installer **inside** Linux.

If Windows asks for a reboot after enabling WSL, reboot and run New WSL again.

## Apply a bundle

**Profiles → Apply** can install Windows apps and then do the New WSL path. Still not a kit restore: no disk remount, no bookmarks.
