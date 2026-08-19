---
title: Kits (Collect / Restore)
description: Snapshot a used PC, then put it back after a Windows reinstall.
order: 5
group: Use
---

## Collect

On the old PC:

1. Run the exe → **Collect**.
2. Pick a destination that is **not the system drive** and not a developer volume you might format.
3. Tick apps (grouped by category). Tick Linux disks and data volumes you want kept.
4. Write the kit. A copy of the exe is placed in the folder.

Never format those data drives. Never delete a Linux distro unless you mean to.

## Restore

Install Windows 11. Leave the data drives alone.

1. Run the exe → **Restore**.
2. Pick the kit.
3. Tick packages. Confirm remounting disks and restoring browser bookmarks.
4. Apply.

Restore remounts the **existing** Linux disk. Tools inside it come back with the disk. Do not run New WSL on top unless you asked to rebuild the toolchain.

Password manager and browser go in first so you can log in while the rest installs. Then open the extensions page and click **Add** for each one.
