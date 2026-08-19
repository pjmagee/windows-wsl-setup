---
title: Profiles
description: Named lists of Windows winget packages and Linux tools, grouped by category.
order: 3
group: Use
---

A profile is a JSON ID list. Categories live on the **catalog**, not on the profile.

Shipped **bundles**:

| Id | Windows | Linux | When |
|---|---|---|---|
| `default` | browsers, Terminal, VS Code, Docker, 1Password, Git | `home` | Fresh PC, no kit |
| `home` | default + games, media, Grok/Claude | `home` (grok, claude, opencode, …) | Home workstation |
| `work` | default + Copilot, no games | `work` (Copilot CLI; no grok/claude) | Work laptop |

Linux categories: environment, sdks, devops, cloud, integrations, agents, content.  
Windows also has browsers, editors, games, media, utils.

## Customise

In the TUI: **Profiles** → tick by category → `s` saves `custom` under `%USERPROFILE%\.windows-wsl-setup\profiles\`.

From the CLI (JSON):

```
windows-wsl-setup profile new my-dev --from home
windows-wsl-setup profile add my-dev --linux kubectl --windows Brave.Brave
windows-wsl-setup apply my-dev
```

`suggest` reads `winget list` and moves SDKs / cloud CLIs onto Linux when the catalog says `preferLinux` (Node, Azure CLI, Go, JDK, …).

Apply **installs**. It never uninstalls Windows apps. Linux `install.sh` may still prune extras that are not on the chosen linux profile.
