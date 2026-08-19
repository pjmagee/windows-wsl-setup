---
title: Welcome
description: What Windows WSL Setup is, and which path to take.
order: 1
group: Start
---

Windows WSL Setup is **one Windows console app**. Download the exe from GitHub Releases. Do not clone this repository unless you are changing it.

Three different jobs. Do not mix them.

| You have | You want | Mode |
|---|---|---|
| A used Windows 11 PC | A kit that survives a reset | **Collect** |
| A kit on a data drive | The same disks and apps back | **Restore** |
| No kit | Ubuntu 26.04 + a toolchain | **New WSL** |
| No kit | A named software list (Windows + Linux) | **Profiles / Apply** |

A **kit** answers “what does this PC have?”  
A **profile** answers “what do I want on a blank machine?”

Restore wins when you still have an `ext4.vhdx`. New WSL and Apply do not remount disks.

## What the exe does not click

- 1Password → Settings → Developer → Use the SSH agent
- Steam → add an existing library folder
- Docker Desktop → WSL integration for your distro
- Brave → **Add to Brave** on the extensions page it opens
