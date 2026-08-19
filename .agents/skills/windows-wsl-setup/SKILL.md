---
name: windows-wsl-setup
description: >
  Collect or restore a Windows 11 PC with Windows WSL Setup (winget kit, Dev Drive, WSL, Brave).
  Use when the user is about to reset Windows, just reinstalled Windows, says Collect/Restore,
  mentions a kit on a data drive, or asks an agent to finish leftovers the exe cannot do.
---

# Windows WSL Setup — agent skill

The product is **`windows-wsl-setup.exe`**. Users download it from this repo’s Releases. They do not clone the repo and they do not run PowerShell for Collect or Restore.

You are the operator for **leftovers and failures**, not a second installer.

## Do this first

1. If the human is on **Windows** and wants backup or restore, run or tell them to run `windows-wsl-setup.exe` (Collect or Restore). Do not invent winget/wsl/diskpart commands unless the exe failed.
2. Never format data drives. Never `wsl --unregister` unless they confirm.
3. Never install Node, Go, Rust, Python, or Azure CLI on Windows to “fix” a missing tool. Those belong in Ubuntu via `./install.sh`.

## When the exe is enough (stay out)

| Job | Who |
|---|---|
| Scan winget / WSL / Dev Drive | Collect |
| Write the kit + winget manifest | Collect |
| Pick packages and `winget install` | Restore |
| Remount Dev Drive, import WSL | Restore |
| Copy Brave bookmarks | Restore |
| Open `extensions.html` | Restore |

## When you are needed

The CLI cannot click other apps’ UIs or approve UAC for the user.

**After Restore (home PC):**

- 1Password → Settings → Developer → Use the SSH agent
- Disable Windows service **OpenSSH Authentication Agent** (1Password owns the named pipe)
- Steam → add the existing library on the games drive
- Docker Desktop → WSL integration → the restored distro (usually Ubuntu-26.04)
- Brave: user clicks **Add to Brave** on each store link (policy install is optional; do not force it unless they ask)
- If WSL starts with **Access Denied** on `ext4.vhdx`: grant `NT VIRTUAL MACHINE\Virtual Machines` (SID `S-1-5-83-0`) Full control, then retry
- If Mount-VHD fails: they need an elevated session / Hyper-V PowerShell

**After WSL actually opens:** the Windows exe does not run the Linux toolchain. Inside Ubuntu-26.04:

```bash
cd ~/code/windows-wsl-setup && git pull && ./install.sh home
```

Use `./install.sh work` only on a work laptop (that path is `windows/bootstrap.ps1`, not this skill’s Collect/Restore).

**Collect-time judgement (only if they ask you in chat instead of using the TUI):**

- Kit destination must not be `C:` and should not be the Dev Drive itself
- Default-off: Docker data VHDX (often huge), Windows copies of language runtimes
- Default-on: browsers, 1Password, editors, game launchers, Steam, WSL distros that are not `docker-desktop`

## Work laptop (not a C: reset)

If they said “set up WSL on this work laptop”, that is **not** Collect/Restore. Follow [AGENTS.md](../../../AGENTS.md) §1 (`windows/bootstrap.ps1` then `./install.sh work`). Do not run Collect.

## Finding a kit

Restore scans `D:`–`Z:\Backups\*\KIT.json`. If none, look at other letters for `KIT.json`. Read `START-HERE.txt` in that folder. Do not move the kit onto `C:`.
