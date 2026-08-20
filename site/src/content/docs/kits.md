---
title: Kits (Collect / Restore)
description: Snapshot a used PC, then put it back after a Windows reinstall.
order: 5
group: Use
---

A kit is a folder with `KIT.json`. Collect writes it. Restore reads it. The destination is never `C:`.

## Collect

On the used PC:

1. Run the exe → **Collect** (or `windows-wsl-setup collect`).
2. Pick a destination that is **not the system drive** and not a developer volume you might format.
3. **Apps** — tick winget packages, grouped by category. Space on a header toggles the whole group. `/` filters.
4. **WSL** — tick Linux disks to keep. Unticked distros are left installed; we never unregister them from Collect.
5. **Host** — Dev Drive VHDX, Docker data, Brave bookmarks + extensions list, Windows dotfiles (`.wslconfig`, Git, SSH, Terminal, PowerShell, Grok config).
6. **Write**. The folder gets `KIT.json`, `START-HERE.txt`, inventory, copied config, Brave `Bookmarks` + `extensions.html`, and a copy of `windows-wsl-setup.exe`.

Never format those data drives. Never `wsl --unregister` unless you mean to delete that disk.

## Restore

Install Windows 11. Leave the data drives alone.

1. Run the exe → **Restore** (or `windows-wsl-setup restore`, optional kit path).
2. The TUI scans `D:`–`Z:\Backups` for `KIT.json`. Pick the kit.
3. Tick packages. Confirm remounting disks and restoring Brave data.
4. Apply. Winget runs in **priority** order: password manager, browser, desktop, then the rest.

Restore remounts the **existing** Linux VHDX. Tools inside it come back with the disk. Do not run New WSL on top unless you asked to rebuild the toolchain.

Brave bookmarks are copied (close Brave first if it is open). `extensions.html` opens so you click **Add** on each Chrome Web Store page — the exe cannot click those.

## After restore

[Getting started](../getting-started/) lists leftovers the exe cannot click: 1Password SSH agent, game libraries, Docker WSL integration, VS Code WSL, and Access Denied on a remounted VHDX.
