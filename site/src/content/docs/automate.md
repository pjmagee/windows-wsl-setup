---
title: Automate
description: JSON CLI. Errors on stderr. wwm spec is OpenCLI.
order: 8
group: Reference
---

These print JSON on stdout. TUI modes (`wwm`, `wwm collect`, `wwm restore`, `wwm profiles` with no subcommand) do not.

```
wwm spec
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
wwm apply home --windows-only
wwm apply blank --linux-only --distro Debian
wwm new-wsl --profile blank --distro Debian
wwm new-wsl --profile home --distro Ubuntu-26.04
wwm distro remove Debian --yes
```

`wwm spec` dumps the [OpenCLI](https://opencli.org/) description (also at `schema/wwm.opencli.json`).

`inventory` dumps a live scan of this PC as JSON (no kit write).

Install the skill first: [Agents](../agents/).
