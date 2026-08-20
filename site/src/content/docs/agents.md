---
title: Agents
description: Install the skill so Grok, Claude, Codex, or Copilot can drive Collect, Restore, and Apply.
order: 3
group: Start
---

The skill teaches an agent to download the exe and run Collect, Restore, New WSL, or Apply. Install it at **user** scope so it works on a used PC and on a fresh Windows 11.

## Install the skill

**GitHub CLI** 2.90+:

```
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent grok
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent claude-code
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent github-copilot
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent codex
```

**skills.sh**:

```
npx skills add pjmagee/windows-wsl-manager --skill windows-wsl-setup -g -y
```

**Claude / Grok plugin marketplace:**

```
/plugin marketplace add pjmagee/windows-wsl-manager
/plugin install windows-wsl-setup@windows-wsl-manager
```

Or ask the agent to read [SKILL.md](https://raw.githubusercontent.com/pjmagee/windows-wsl-manager/main/skills/windows-wsl-setup/SKILL.md).

## Drive

1. Confirm Windows (not WSL, not Git Bash).
2. Install `wwm.exe` into `~\.wwm` with the PowerShell block above.
3. **Collect** on the used PC. **Restore** on the new PC. **New WSL** or **apply** if there is no kit.
4. Password manager first, then Brave.
5. Do not format data drives. Do not unregister a distro unless the human confirms.

```
wwm suggest
wwm map <winget-id>
wwm profile list
wwm apply default
```

Leftovers: [Getting started](../getting-started/). Command list: [Automate](../automate/).
