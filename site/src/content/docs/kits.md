---
title: Kits
description: Collect Windows software and disks. Restore them on a new PC.
order: 4
group: Use
---

A kit is a folder with `KIT.json`. You build it on a used PC. You restore it after a clean Windows 11 install. Never write it on the system drive.

This is the “I am about to reset Windows” path.

## Collect

```
wwm collect
```

1. Destination is a data drive, not `C:`.
2. **Apps** — tick winget packages by category. Space on a header toggles the group. `/` filters.
3. **WSL** — tick Linux disks to copy into the kit. Unticked distros are left alone on this PC.
4. **Host** — optional: Dev Drive, Docker data, Brave bookmarks and extensions, Windows config (`.wslconfig`, Git, SSH, Terminal, PowerShell).
5. **Write**. The folder gets `KIT.json`, `START-HERE.txt`, inventory, copied files, and a copy of `wwm.exe`.

## Restore

Install Windows 11. Do not format the data drives.

```
wwm restore
wwm restore D:\Backups\my-kit
```

1. The TUI looks for `KIT.json` under `D:`–`Z:\Backups` if you omit the path.
2. Tick packages. Confirm remounting disks and restoring Brave data.
3. Apply. Winget runs in **priority** order: password manager, browser, desktop, editors, Git, Docker, then the rest.

Restore **remounts** a collected WSL VHDX. Use [New WSL](../wsl/) only when there is no disk to bring back.

Brave bookmarks are copied (close Brave first). `extensions.html` opens so you can click **Add**.

## Leftovers

- **1Password** — sign in, **Use the SSH agent**. Windows OpenSSH Authentication Agent **off**.
- **Brave** — **Add** each extension.
- **Steam / Epic** — existing library folder.
- **Docker Desktop** — WSL integration for the distro you restored.
- **Access Denied** on a VHDX — grant SID `S-1-5-83-0` Full control.

<p class="note">Do not format those data drives. Do not <code>wsl --unregister</code> unless you intend to delete that disk.</p>
