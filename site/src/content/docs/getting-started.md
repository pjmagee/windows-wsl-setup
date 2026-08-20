---
title: Getting started
description: Collect a kit. Restore existing WSL instances or provision new ones.
order: 1
group: Start
---

| You have | You want | Run |
|---|---|---|
| A used Windows 11 PC | A kit that survives a reinstall | **Collect** |
| That kit on a data drive | The same apps and disks on fresh Windows 11 | **Restore** |
| No kit | A new Linux environment | **New WSL** |
| A named software list | Winget packages and Linux CLIs | **Profiles** |

**Collect** writes a kit. **Restore** remounts its WSL VHDX. **New WSL** provisions a distro. **Profiles** apply a list.

```
wwm
wwm collect
wwm restore
wwm new-wsl
wwm profiles
```

If Windows just enabled WSL, reboot when asked and run `wwm` again.

Prefer an agent? [Install the skill](../agents/).

## Paths

**Collect** — destination must not be the system drive. Tick winget apps, WSL disks, and host leftovers (Dev Drive, Docker data, Brave, Windows dotfiles). The kit folder contains KIT.json, a copy of the exe, and START-HERE.txt.

**Restore** — leave data drives intact. The TUI finds KIT.json in a Backups folder on a data drive. Tick packages, remount disks, Apply. Brave bookmarks are copied. extensions.html opens for extensions.

**New WSL** — Ubuntu, Debian, or Arch, plus a linux profile. Enables WSL, installs the distro, auto-configures passwordless sudo, and applies the profile. Homebrew installations follow.

**Profiles** — tick Windows and Linux packages, optionally create WSL, Apply. Suggest from this PC (`g`). Save a user profile (`s`).

Details: [Kits](../kits/), [Profiles](../profiles/), [WSL](../wsl/).

## After Restore or New WSL

- **1Password** — sign in, enable **Use the SSH agent**. Turn the Windows OpenSSH Authentication Agent service **off**.
- **Brave** — **Add** each extension from extensions.html.
- **Steam / Epic** — point at the existing library folder.
- **Docker Desktop** — enable WSL integration for the distro you restored or created.
- **VS Code** — WSL extension; open a Linux path.
- **Access Denied** on a remounted VHDX — grant SID `S-1-5-83-0` Full control.

<p class="note">Do not format data drives. Do not <code>wsl --unregister</code> unless you intend to delete that disk.</p>
