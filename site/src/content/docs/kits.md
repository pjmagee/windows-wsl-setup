---
title: Kits (Collect / Restore)
description: Snapshot a used PC onto a data drive, then remount disks after a Windows reset.
order: 4
group: Use
---

## Collect

On the old PC:

1. Run the exe → **Collect**.
2. Pick a destination that is **not C:** and not the Dev Drive. Suggested path: `<letter>:\Backups\<HOST>-<date>\`.
3. Tick winget apps (grouped by category). Space on a category header toggles the group.
4. Tick WSL distros and the Dev Drive if you use them.
5. Write the kit. Keep that folder. A copy of the exe is placed in it.

Never format data drives. Never `wsl --unregister` unless you mean to delete that Linux disk.

## Restore

Install Windows 11. Do **not** wipe the data drives.

1. Run the exe (from Releases, or the copy in the kit) → **Restore**.
2. Pick the kit.
3. Tick packages, remount Dev Drive, restore WSL, restore Brave bookmarks.
4. Apply.

Restore remounts the **existing** VHDX. Linux tools come back with that disk. Do not run New WSL on top unless you asked to rebuild the toolchain.

If WSL returns Access Denied on `ext4.vhdx` after import-in-place, grant SID `S-1-5-83-0` (NT VIRTUAL MACHINE\Virtual Machines) Full control. That SID is the Hyper-V VM group the new machine uses.
