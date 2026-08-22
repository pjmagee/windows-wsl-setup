---
title: WSL
description: Backup and restore Linux disks, or create a new official distro.
order: 5
group: Use
---

`wwm` is still a Windows program. It talks to `wsl.exe`.

## Backup

**Collect** copies each ticked distro’s VHDX into the kit. That is the backup.

## Restore a disk

**Restore** imports that VHDX (`wsl --import-in-place`). Distros already on the machine stay installed.

```
wwm restore
```

Do not run New WSL if the kit already has the disk you want.

## New empty distro

```
wwm new-wsl --profile blank --distro Debian
```

Installs that distro (must still be in `wsl --list --online`), creates the Linux user, and turns on passwordless sudo. No extra packages. JSON looks like:

```
[
  { "step": "distro", "ok": true, "detail": "Debian installed." },
  { "step": "user", "ok": true, "detail": "created alex (uid 1000, empty password)" },
  { "step": "sudo", "ok": true, "detail": "ok user=alex" },
  { "step": "linux", "ok": true, "detail": "blank: distro + passwordless sudo only" }
]
```

The new distro sits **beside** whatever is already the WSL default. It does not become default.

## New distro with packages

```
wwm new-wsl --profile home --distro Ubuntu-26.04
wwm apply work --linux-only --distro Debian
```

Same host steps as `blank`, then the Linux package list for that profile (Homebrew CLIs).

`home` includes grok, claude, opencode, hugo, stripe. `work` is Copilot CLI plus the shared toolchain.

## Remove

```
wwm distro remove Debian --yes
```

Unregisters the distro (deletes the disk). Requires `--yes`.

x86_64 only. Create ids: [Distros](../distros/). Disk on another drive: `--location` / `wwm distro move`.
