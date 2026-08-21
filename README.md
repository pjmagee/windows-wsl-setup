# Windows WSL Manager

Export a messy Windows 11 PC. Restore it after a clean install. Or stand up Linux from a software profile.

Download `wwm.exe` from **Releases**, or PowerShell:

```
irm https://pjmagee.github.io/wwm/install.txt | iex
```

Do not clone this repo to use it. Site: <https://pjmagee.github.io/wwm/>

## Agent-ready

The skill id is `wwm-cli`. Pick **one** installer.

GitHub CLI (this one is Grok only — pass `--agent claude-code`, `github-copilot`, or `codex` for those):

```
gh skill install pjmagee/wwm wwm-cli --scope user --agent grok
```

skills.sh (any detected agent; needs Node):

```
npx skills add pjmagee/wwm --skill wwm-cli -g -y
```

Then ask the agent to collect, restore, or apply a profile. The skill tells it how to download the exe.

## Paths

1. **Collect** on the used PC (kit on a non-system drive).
2. **Restore** on fresh Windows 11 — apps first (password manager, then your browser), then remount disks.
3. **New WSL** if there is no Linux disk to bring back. Pick a distro + a profile.
4. **Profiles** when you want a named software list instead of a snapshot.

```
wwm
wwm collect
wwm restore
wwm new-wsl
wwm new-wsl --profile blank --distro Debian
wwm distro move Debian D:\WSL\Debian
wwm distro clone Ubuntu-26.04 Ubuntu-dev --location D:\WSL\Ubuntu-dev
wwm distro sync
wwm distro remove Debian --yes
wwm spec
wwm profiles
```
