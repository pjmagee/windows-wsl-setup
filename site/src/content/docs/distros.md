---
title: Distros
description: Official Linux targets, and what we bootstrap on each.
order: 7
group: Reference
---

| Situation | Distro |
|---|---|
| Restore a kit | Whatever disk you collected. We do not convert it. |
| New WSL / Apply linux | **Ubuntu 26.04** (default), **Debian**, or **Arch** (`archlinux`) |
| Other distros already installed | Left alone. We never unregister them. |

CLIs (uv, bun, rust, git extras, cloud tools) come from **Homebrew** on every supported distro. Only the bootstrap set uses the distro package manager (`apt` on Ubuntu/Debian, `pacman` on Arch).

| Distro | WSL name | Package manager | Notes |
|---|---|---|---|
| Ubuntu 26.04 LTS | `Ubuntu-26.04` | apt | Default. Best Homebrew-on-Linux fit. |
| Debian | `Debian` | apt | Same bootstrap as Ubuntu. |
| Arch | `archlinux` | pacman | amd64 only. First boot is often root until we create a user. |

Fedora, Kali, and openSUSE stay **unsupported** as new-instance targets (different package managers, or the wrong job). You can still **restore** a kit that already has them.

`uname -m` should be `x86_64` for Arch. Ubuntu and Debian also have ARM images; Homebrew is most reliable on Ubuntu.
