---
title: WSL
description: Restore existing WSL instances, or provision Ubuntu, Debian, or Arch.
order: 6
group: Use
---

## Restore existing WSL instances

If the kit includes a Linux disk, Restore imports that VHDX. Other distros on the machine stay installed.

## New WSL

Use this when there is no disk to restore.

1. Pick Ubuntu, Debian, or Arch, then a linux profile (`blank`, `home`, or `work`). `blank` is the distro plus a passwordless-sudo user — no Homebrew.
2. Enables WSL if needed, installs the distro, creates the Linux user, auto-configures passwordless sudo, and applies the profile. Homebrew installations follow.
3. Windows Terminal already gets an official tab from Microsoft.WSL. WWM does not overwrite that command line. It adds a **wsl** launcher only. Does not change the WSL / Terminal default if another distro is already default.
4. `wwm distro remove <name> --yes` unregisters the distro (the official Terminal tab goes with it).

Requires x86_64. Fedora, Kali, and openSUSE are restore-only.

## Apply

**Profiles → Apply** installs Windows packages, then can run New WSL. It does not remount a VHDX.

Linux **work** is Copilot CLI plus the shared toolchain. Linux **home** keeps grok, claude, opencode, hugo, and stripe.

Official targets: [Distros](../distros/).
