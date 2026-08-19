---
title: Automate
description: Drive the exe from a terminal or an agent. JSON on stdout.
order: 8
group: Reference
---

These commands always print JSON. Errors go to stderr.

```
windows-wsl-setup catalog linux
windows-wsl-setup catalog windows
windows-wsl-setup profile list
windows-wsl-setup profile show home
windows-wsl-setup profile new my-dev --from home
windows-wsl-setup profile add my-dev --linux kubectl --windows Google.Chrome
windows-wsl-setup search linux kubectl
windows-wsl-setup search winget terraform
windows-wsl-setup map Microsoft.AzureCLI
windows-wsl-setup suggest
windows-wsl-setup apply home
windows-wsl-setup apply home --windows-only
windows-wsl-setup apply home --linux-only
windows-wsl-setup new-wsl --profile home
```

Install the skill first: [Agents](../agents/).

`apply` installs Windows packages in **priority order** (password manager, browser, desktop, then the rest), then optionally creates the selected Linux distro and runs the linux profile.
