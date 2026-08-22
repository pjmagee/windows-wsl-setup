---
title: Distros
description: Latest official image per family. Pengwin is manage-only.
order: 7
group: Reference
---

New WSL creates **one** id per family: the latest name still in `wsl --list --online`. Ubuntu stays pinned to 26.04. x86_64 only.

| Family | Create id (today) | Alias | Bootstrap |
|---|---|---|---|
| Ubuntu | Ubuntu-26.04 | ubuntu | apt |
| Debian | Debian | debian | apt |
| Arch | archlinux | arch | pacman |
| Kali | kali-linux | kali | apt |
| Fedora | FedoraLinux-44 | fedora | dnf |
| AlmaLinux | AlmaLinux-10 | alma | dnf |
| openSUSE | openSUSE-Tumbleweed | opensuse | zypper |
| Oracle | OracleLinux_9_5 | oracle | dnf |

`wwm distros` shows the resolved ids for this PC. A name missing from `wsl --list --online` is hidden. Fedora and Alma pick the **highest** versioned name online (skip Alma Kitten).

Not create targets: older numbered Ubuntu/Alma/Fedora/Oracle, openSUSE Leap, SUSE Linux Enterprise, **Pengwin**, Fedora Remix. Pengwin is a paid Store app (Whitewater Foundry), not in Microsoft’s online list. If it is already installed, Collect / Restore / `distro sync` / `distro remove` / `distro move` / `distro clone` still work.

```
wwm distros
wwm new-wsl --profile blank --distro fedora --location D:\WSL\Fedora
wwm distro move Debian D:\WSL\Debian
wwm distro clone Ubuntu-26.04 Ubuntu-dev --location D:\WSL\Ubuntu-dev
wwm distro remove Debian --yes
```

