---
title: Distros
description: We support one empty-Linux target. Restored disks keep whatever you already had.
order: 6
group: Reference
---

| Situation | Distro |
|---|---|
| Restore a kit | Whatever VHDX you collected. We do not convert it. |
| New WSL / Apply linux | **Ubuntu 26.04** only |
| Leftover `Ubuntu`, `Ubuntu-24.04`, `docker-desktop` | Left installed. We never unregister them. |

We do **not** offer Fedora, Arch, or “pick a package manager”.

On a new instance:

- **apt** — system packages (`packages/apt.txt`: git, jq, build-essential, …)
- **Homebrew** — CLIs and language runtimes (`uv`, `bun`, `fnm`, rust, go, dotnet, cloud CLIs, …)

`uname -m` must be `x86_64`. Homebrew bottles and Compass are amd64. Stop on ARM.

Ubuntu 26.04’s image wants WSL **2.4.10+**. The exe runs `wsl --update` first. If `Ubuntu-26.04` is missing from `wsl --list --online`, update WSL elevated and retry. We will not install a different Ubuntu.
