# Windows WSL Setup

Export a messy Windows 11 PC. Restore it after a clean install. Or stand up Linux (Ubuntu, Debian, or Arch) from a software profile.

Download `windows-wsl-setup.exe` from **Releases**. Do not clone this repo to use it.

Site: <https://pjmagee.github.io/windows-wsl-manager/>

## Agent-ready

```
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent grok
npx skills add pjmagee/windows-wsl-manager --skill windows-wsl-setup -g -y
```

Then ask Grok, Claude, Codex, or Copilot to collect, restore, or apply a profile. The skill tells them how to download the exe and which commands exist.

## Paths

1. **Collect** on the used PC (kit on a non-system drive).
2. **Restore** on fresh Windows 11 — apps first (password manager, then your browser), then remount disks.
3. **New WSL** if there is no Linux disk to bring back. Pick Ubuntu, Debian, or Arch + a profile.
4. **Profiles** when you want a named software list instead of a snapshot.

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
```
