---
title: Kits
description: Collect a kit and restore it on a new installation of Windows.
order: 5
group: Use
---

A kit is a folder with `KIT.json`. Collect writes it. Restore reads it. Never on the system drive.

## Collect

1. Run the exe → **Collect**, or `wwm collect`.
2. Pick a destination that is not the system drive.
3. **Apps** — tick winget packages by category. Space on a header toggles the group. `/` filters.
4. **WSL** — tick Linux disks to keep. Unticked distros stay installed.
5. **Host** — Dev Drive, Docker data, Brave bookmarks and extensions, Windows dotfiles (`.wslconfig`, Git, SSH, Terminal, PowerShell, Grok).
6. **Write**. The folder gets `KIT.json`, `START-HERE.txt`, inventory, copied config, Brave data, and a copy of the exe.

<p class="note">Do not format those data drives. Do not <code>wsl --unregister</code> unless you intend to delete that disk.</p>

## Restore

Install Windows 11. Leave the data drives alone.

1. Run the exe → **Restore**, or `wwm restore` (optional kit path).
2. The TUI scans `D:`–`Z:\Backups` for `KIT.json`. Pick the kit.
3. Tick packages. Confirm remounting disks and restoring Brave data.
4. Apply. Winget runs in **priority** order: password manager, browser, desktop, then the rest.

Restore remounts the existing WSL VHDX. Do not run New WSL unless you want a new distro instead of that disk.

Brave bookmarks are copied (close Brave first if it is open). `extensions.html` opens so you can **Add** each extension.

## After restore

[Getting started](../getting-started/) lists the leftover clicks: 1Password SSH agent, game libraries, Docker WSL integration, VS Code WSL, and Access Denied on a remounted VHDX.
