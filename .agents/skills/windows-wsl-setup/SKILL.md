---
name: windows-wsl-setup
description: >
  Collect or restore a Windows 11 PC with Windows WSL Setup, create Ubuntu 26.04,
  or build/apply a software profile (winget + WSL). Use when the user is about
  to reset Windows, just reinstalled, wants a new empty WSL, wants a default or
  custom profile, or asks to finish leftovers the exe cannot click
  (1Password, Steam, Docker, Brave Add).
---

# Windows WSL Setup — agent skill

The product is **`windows-wsl-setup.exe`** from this repo’s Releases.

Users do **not** clone the repo. They do **not** run `install.sh`, PowerShell, or `bootstrap.ps1` unless they already cloned and asked.

| Mode | When |
|---|---|
| **Collect** | Used PC, about to reset. Writes a kit on a non-C: drive. |
| **Restore** | Fresh Windows 11. Reads the kit. Remounts the **existing** WSL disk. |
| **New WSL** | No kit / empty Ubuntu. Installs **Ubuntu 26.04 only** + a linux profile. |
| **Profiles / Apply** | No kit. Named software lists. Shipped `default` / `home` / `work`, or custom. |

Do not offer Fedora, Arch, or another Ubuntu. Do not invent a second installer.

A **kit** is “what this PC has”. A **profile** is “what I want”. Restore wins when they have a VHDX. Apply does not remount disks.

Do **not** run New WSL or Apply-linux on top of a restored distro unless they asked to rebuild tools.

## Do this first

1. Point them at `windows-wsl-setup.exe`.
2. Never format data drives. Never `wsl --unregister` unless they confirm.
3. If `wsl --install` asked for a reboot, reboot and run New WSL / Apply again.

## Build a profile from a messy Windows box

CLI always prints JSON (stderr for errors). Confirm winget ids with the user before adding.

```
windows-wsl-setup suggest
windows-wsl-setup map <winget-id>
windows-wsl-setup search winget <query>
windows-wsl-setup search linux <query>
windows-wsl-setup catalog linux
windows-wsl-setup catalog windows
windows-wsl-setup profile list
windows-wsl-setup profile new <id> --from home
windows-wsl-setup profile add <id> --linux <tool> --windows <Winget.Id>
windows-wsl-setup profile remove <id> --linux <tool>
windows-wsl-setup apply <id>
windows-wsl-setup apply <id> --windows-only
windows-wsl-setup apply <id> --linux-only
```

Rules:

- Prefer Linux for SDKs and cloud CLIs (`preferLinux` on the Windows catalog, or `map`).
- Do not add Windows Node / Go / Rust / Azure CLI / JDK when a linux equivalent exists.
- Games, browsers, editors, 1Password, Docker Desktop stay on Windows.
- Custom profiles write to `%USERPROFILE%\.windows-wsl-setup\profiles\`.
- `suggest` does not save. `profile save` / `profile new` does.

## After Restore — leftovers the exe cannot click

- 1Password → Settings → Developer → Use the SSH agent
- OpenSSH Authentication Agent Windows service **off**
- Steam → add the existing library folder
- Docker Desktop → WSL integration for the restored distro
- Brave: **Add to Brave** on the extensions page the exe opened
- If WSL is Access Denied on `ext4.vhdx`: grant SID `S-1-5-83-0` Full control
- If Dev Drive did not mount: they need UAC / Hyper-V PowerShell

## Work laptop (new empty Ubuntu)

`windows-wsl-setup.exe` → **New WSL** → **work**, or `apply work --linux-only`. Copilot that already cloned this repo may still follow [AGENTS.md](../../../AGENTS.md) §1 (`bootstrap.ps1`).
