---
title: Download
description: Get windows-wsl-setup.exe and run it. No clone, no PowerShell.
order: 2
group: Start
---

1. Open [Releases](https://github.com/pjmagee/windows-wsl-setup/releases).
2. Download `windows-wsl-setup.exe`.
3. Run it. The home menu is **Collect / Restore / New WSL / Profiles**.

That is the whole install. There is no MSI, no store package, no `git clone`.

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
```

CLI verbs that print JSON (for agents) are documented under [Automate](../automate/).

If Windows just enabled WSL, it may ask for a **reboot**. Reboot, then run New WSL or Apply again.
