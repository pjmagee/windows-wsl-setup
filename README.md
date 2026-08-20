# Windows WSL Manager

Export a messy Windows 11 PC. Restore it after a clean install. Or stand up Linux (Ubuntu, Debian, or Arch) from a software profile.

Download `wwm.exe` from **Releases**, or PowerShell:

```
New-Item $HOME\.wwm -ItemType Directory -Force | Out-Null
Invoke-WebRequest -UseBasicParsing https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/wwm.exe -OutFile $HOME\.wwm\wwm.exe
$env:Path = "$HOME\.wwm;$env:Path"
wwm
```

Do not clone this repo to use it. Site: <https://pjmagee.github.io/windows-wsl-manager/>

## Agent-ready

The skill id is `windows-wsl-setup`. Pick **one** installer.

GitHub CLI (this one is Grok only — pass `--agent claude-code`, `github-copilot`, or `codex` for those):

```
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent grok
```

skills.sh (any detected agent; needs Node):

```
npx skills add pjmagee/windows-wsl-manager --skill windows-wsl-setup -g -y
```

Then ask the agent to collect, restore, or apply a profile. The skill tells it how to download the exe.

## Paths

1. **Collect** on the used PC (kit on a non-system drive).
2. **Restore** on fresh Windows 11 — apps first (password manager, then your browser), then remount disks.
3. **New WSL** if there is no Linux disk to bring back. Pick Ubuntu, Debian, or Arch + a profile.
4. **Profiles** when you want a named software list instead of a snapshot.

```
wwm
wwm collect
wwm restore
wwm new-wsl
wwm profiles
```
