---
title: Profiles
description: Named Windows and Linux software lists. Apply installs.
order: 4
group: Use
---

A profile is a list of package ids. Categories live on the catalog.

Shipped bundles (Windows + Linux + Ubuntu):

| Id | Windows | Linux |
|---|---|---|
| blank | none | none (distro + passwordless sudo only) |
| default | Terminal, PowerShell, PowerToys, Brave, 1Password, VS Code, Docker Desktop, Git | home |
| home | default + Grok, Claude, Cursor, Steam, Epic, VLC, Discord | home |
| work | default + GitHub Copilot | work |

Linux **home** includes grok-build, claude-code, opencode, hugo, and stripe. Linux **work** is Copilot CLI plus the shared toolchain; home-only agents are pruned.

## Customise

In the TUI, tick packages by category. Save (`s`), delete (`d`), or suggest (`g`). `"Media PC"` is stored as `media-pc`.

```
wwm profile new "Media PC" --from home
wwm profile new my-dev --from home --name "My Dev"
wwm profile add my-dev --linux kubectl --windows Google.Chrome
wwm apply media-pc
wwm profile delete media-pc
```

`suggest` reads what is installed and maps SDKs / cloud CLIs onto Linux when the catalog says `preferLinux` (Azure CLI → `azure-cli` in WSL).

Apply installs. It does not uninstall Windows apps. Linux setup may drop CLIs that are not on the chosen linux profile.

Winget order is **priority**: password manager, browser, desktop, editors, Git, Docker, agents, then the rest.
