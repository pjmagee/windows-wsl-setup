---
title: Automate
description: JSON CLI for catalog, profiles, suggest, apply, and New WSL.
order: 8
group: Reference
---

These commands print JSON. Errors go to stderr. TUI modes do not.

```
wwm catalog linux
wwm catalog windows
wwm distros
wwm profile list
wwm profile show home
wwm profile new my-dev --from home --name "My Dev"
wwm profile add my-dev --linux kubectl --windows Google.Chrome
wwm profile delete my-dev
wwm search linux kubectl
wwm search winget terraform
wwm map Microsoft.AzureCLI
wwm suggest
wwm apply home
wwm apply home --windows-only
wwm apply home --linux-only
wwm new-wsl --profile home
wwm new-wsl --profile home --distro Debian
```

`inventory` dumps a live scan of this PC as JSON.

Install the skill first: [Agents](../agents/).

`apply` installs Windows packages in **priority** order, then can create the selected Linux distro and run the linux profile. It does not uninstall Windows apps.
