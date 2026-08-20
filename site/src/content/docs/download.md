---
title: Download
description: Get the Windows binary. Do not clone the repo.
order: 2
group: Start
---

1. Open [Releases](https://github.com/pjmagee/windows-wsl-manager/releases).
2. Download `windows-wsl-setup.exe`.
3. Run it. Home screen: **Collect**, **Restore**, **New WSL**, **Profiles**.

That filename is the current release artifact of **Windows WSL Manager**. There is no MSI. Agents download the same file from:

`https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/windows-wsl-setup.exe`

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
```

JSON commands (catalog, profile, suggest, apply, map, search, distros) are listed under [Automate](../automate/). First-run leftovers and which mode to pick: [Getting started](../getting-started/).

If Windows just enabled WSL, reboot when asked, then run the exe again.
