---
name: windows-wsl-setup
description: >
  Collect or restore a Windows 11 PC with Windows WSL Setup.
  Use when the user is about to reset Windows, just reinstalled Windows, or
  asks to finish leftovers the exe cannot click (1Password, Steam, Docker, Brave Add).
---

# Windows WSL Setup — agent skill

The product is **`windows-wsl-setup.exe`** from this repo’s Releases.

Users do **not** clone the repo. They do **not** run `install.sh`, PowerShell, or `bootstrap.ps1` for a home PC reset.

Restoring WSL imports the **existing** Linux disk. Homebrew/Node/Git come back with that disk. Do not tell them to clone this repo or run `./install.sh` unless they explicitly want to **rebuild** an empty Ubuntu.

## Do this first

1. Run or point them at `windows-wsl-setup.exe` → Collect (old PC) or Restore (new PC).
2. Never format data drives. Never `wsl --unregister` unless they confirm.
3. Do not invent a second winget/wsl installer.

## After Restore — only leftovers the exe cannot click

- 1Password → Settings → Developer → Use the SSH agent
- OpenSSH Authentication Agent Windows service **off**
- Steam → add the existing library folder
- Docker Desktop → WSL integration for the restored distro
- Brave: **Add to Brave** on the extensions page the exe opened
- If WSL is Access Denied on `ext4.vhdx`: grant SID `S-1-5-83-0` Full control
- If Dev Drive did not mount: they need UAC / Hyper-V PowerShell

## Work laptop (new empty Ubuntu, not a C: reset)

That is a different job: [AGENTS.md](../../../AGENTS.md) §1.
