---
title: Automate
description: Drive the exe from a terminal or an agent. JSON on stdout, errors on stderr.
order: 7
group: Reference
---

No `--json` flag. These commands always print JSON.

```
windows-wsl-setup catalog linux
windows-wsl-setup catalog windows
windows-wsl-setup profile list
windows-wsl-setup profile show home
windows-wsl-setup profile new my-dev --from home
windows-wsl-setup profile add my-dev --linux kubectl --windows Microsoft.VisualStudioCode
windows-wsl-setup profile remove my-dev --linux hugo
windows-wsl-setup search linux kubectl
windows-wsl-setup search winget terraform
windows-wsl-setup map Microsoft.AzureCLI
windows-wsl-setup suggest
windows-wsl-setup apply home
windows-wsl-setup apply home --windows-only
windows-wsl-setup apply home --linux-only
windows-wsl-setup new-wsl --profile home
```

## Agent sequence (messy Windows → WSL)

1. `suggest` — draft from `winget list`. Does not write.
2. `map <id>` — confirm Linux equivalents (`preferLinux`).
3. `search winget` / `search linux` — fill gaps. Confirm exact winget ids with the human.
4. `profile new` / `profile add` — save under `%USERPROFILE%\.windows-wsl-setup\profiles\`.
5. `apply <id>` — winget install + optional New WSL.

Do not format data drives. Do not `wsl --unregister` unless they confirm. Prefer Linux for SDKs and cloud CLIs. Games, browsers, editors, 1Password, and Docker Desktop stay on Windows.

Custom linux profiles are copied into Ubuntu (`~/.config/wsl-setup/profiles/<id>.json`) before `install.sh <id>`. If that copy fails, Apply stops instead of silently running shipped `home`.
