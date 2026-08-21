---
title: Distros
description: Ubuntu-26.04, Debian, archlinux.
order: 7
group: Reference
---

New WSL can create these (must still be in `wsl --list --online`):

| Distro | WSL name | Package bootstrap |
|---|---|---|
| Ubuntu | Ubuntu-26.04 | apt |
| Debian | Debian | apt |
| Arch | archlinux | pacman |

```
wwm distros
```

JSON includes `choices` (supported ∩ online, plus `installed`), `default`, and `installed` (everything `wsl -l` reports, including `docker-desktop`).

Fedora, Kali, and openSUSE are restore-only. x86_64 only.

```
wwm distro remove Debian --yes
```
