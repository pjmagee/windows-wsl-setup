---
title: Profiles
description: Named lists of Windows apps and Linux tools, grouped by category.
order: 4
group: Use
---

A profile is a list of package ids. Categories live on the catalog, not on the profile.

Shipped **bundles**:

| Id | Windows | Linux | When |
|---|---|---|---|
| `default` | Terminal, your browser, password manager, editor, Git | `home` | Fresh PC, no kit |
| `home` | default + games, media, extra agents | `home` | Home workstation |
| `work` | default + Copilot, no games | `work` | Work laptop |

## Customise

TUI: **Profiles** → tick by category → `s` saves `custom`.

CLI:

```
windows-wsl-setup profile new my-dev --from home
windows-wsl-setup profile add my-dev --linux kubectl --windows Google.Chrome
windows-wsl-setup apply my-dev
```

`suggest` reads what is already installed and moves SDKs / cloud CLIs onto Linux when the catalog says so.

Apply **installs**. It never uninstalls Windows apps. Linux setup may drop extras that are not on the chosen linux profile.

Winget install order is **priority**, not category: password manager → browser → desktop → editors → Git → Docker → agents → later (games, media, cleaners).
