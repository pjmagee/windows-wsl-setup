---
title: Profiles
description: Named Windows (winget) and Linux (WSL) software lists.
order: 6
group: Use
---

A **bundle** points at a Windows list and a Linux list. Categories live on the catalog, not on the profile.

Use this when you want the same software set on more than one machine, or different sets on different distros.

## Shipped

| Id | Windows (winget) | Linux (WSL) |
|---|---|---|
| blank | none | none (distro + passwordless sudo) |
| default | Terminal, PowerShell, PowerToys, Brave, 1Password, VS Code, Docker Desktop, Git | home |
| home | default + Grok, Claude, Cursor, Steam, Epic, VLC, Discord | home |
| work | default + GitHub Copilot | work |

Linux **home** includes grok-build, claude-code, opencode, hugo, stripe. Linux **work** is Copilot CLI plus the shared toolchain.

## Edit

TUI: tick by category, save (`s`), delete (`d`), suggest (`g`). `"Media PC"` is stored as `media-pc`.

```
wwm profile list
wwm profile show home
wwm profile new media-pc --from home --name "Media PC"
wwm profile add media-pc --linux kubectl --windows Valve.Steam
wwm apply media-pc
wwm profile delete media-pc
```

`suggest` reads what is installed. SDKs and cloud CLIs move to Linux when the catalog has `preferLinux` (Azure CLI → `azure-cli` in WSL).

```
wwm map Microsoft.AzureCLI
```

```
{
  "windows": "Microsoft.AzureCLI",
  "category": "cloud",
  "preferLinux": true,
  "linux": "azure-cli"
}
```

## Apply

```
wwm apply home
wwm apply home --windows-only
wwm apply blank --linux-only --distro Debian
```

Windows installs go through winget in **priority** order. Linux may create the distro. Apply does not remount a VHDX and does not uninstall Windows apps.
