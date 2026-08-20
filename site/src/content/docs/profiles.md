---
title: Profiles
description: Named lists of Windows apps and Linux tools. Apply installs; it never uninstalls Windows apps.
order: 4
group: Use
---

A profile is a list of package ids. Categories live on the catalog, not on the profile.

Shipped **bundles** (Windows list + linux list + default distro `Ubuntu-26.04`):

| Id | Windows | Linux | When |
|---|---|---|---|
| `default` | Terminal, PowerShell, PowerToys, Oh My Posh, Brave, 1Password, VS Code, Docker Desktop, Git, 7-Zip | `home` | Fresh PC, no kit |
| `home` | default + Grok, Claude, Cursor, Steam, Epic, VLC, Discord, CCleaner, SteelSeries | `home` | Home workstation |
| `work` | default + GitHub Copilot; no games or home agents | `work` | Work laptop |

Linux `home` includes grok-build, claude-code, opencode, hugo, stripe, and similar. Linux `work` is Copilot CLI plus the shared shell/runtime/cloud set; `install.sh work` prunes the home-only agents.

## Customise

TUI: **Profiles** → tick by category → `s` → type a name → Enter. `"Media PC"` is saved as id `media-pc` under `%USERPROFILE%\.windows-wsl-setup\profiles`. `d` deletes a **user** profile (Enter to confirm). Shipped `home` / `work` / `default` stay. `g` runs **suggest**.

CLI:

```
windows-wsl-setup profile new "Media PC" --from home
windows-wsl-setup profile new my-dev --from home --name "My Dev"
windows-wsl-setup profile add my-dev --linux kubectl --windows Google.Chrome
windows-wsl-setup apply media-pc
windows-wsl-setup profile delete media-pc
```

`suggest` reads what is already installed and moves SDKs / cloud CLIs onto Linux when the catalog says `preferLinux` (for example Azure CLI → `azure-cli` in WSL).

Apply **installs**. It never uninstalls Windows apps. Linux setup may drop extras that are not on the chosen linux profile.

Winget install order is **priority**, not category: password manager → browser → desktop → editors → Git → Docker → agents → later (games, media, cleaners).
