---
title: Agents
description: Install the skill so Grok, Claude, Codex, or Copilot can drive the manager.
order: 3
group: Start
---

The product is **agent-ready**. The skill teaches an agent to download the exe, pick Collect / Restore / New WSL / Apply, and talk JSON.

You do **not** clone this repo to use it. Install the skill at **user** scope so it works on a messy PC and on a fresh Windows 11.

The skill id is still `windows-wsl-setup`. The GitHub repo is `pjmagee/windows-wsl-manager`.

## Install the skill

**GitHub CLI** (preview, `gh` ≥ 2.90):

```
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent grok
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent claude-code
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent github-copilot
gh skill install pjmagee/windows-wsl-manager windows-wsl-setup --scope user --agent codex
```

**skills.sh** (many agents at once):

```
npx skills add pjmagee/windows-wsl-manager --skill windows-wsl-setup -g -y
```

**Claude / Grok plugin marketplace:**

```
/plugin marketplace add pjmagee/windows-wsl-manager
/plugin install windows-wsl-setup@windows-wsl-manager
```

**Paste only:** ask the agent to read  
https://raw.githubusercontent.com/pjmagee/windows-wsl-manager/main/skills/windows-wsl-setup/SKILL.md

The canonical skill folder is `skills/windows-wsl-setup/`. A copy also lives in `.agents/skills/` so in-repo agents see it without installing.

## What the agent should do

1. Confirm you are on Windows (not inside Linux, not Git Bash).
2. Download  
   `https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/windows-wsl-setup.exe`  
   If that 404s, stop — do not clone as a substitute.
3. **Collect** on the used PC (kit on a non-system drive).
4. **Restore** on the new PC, or **New WSL** / **apply** if there is no kit.
5. Password manager first, then Brave, then the rest.
6. Never format data drives. Never unregister a Linux distro unless the human confirms.

```
windows-wsl-setup suggest
windows-wsl-setup map <winget-id>
windows-wsl-setup profile list
windows-wsl-setup apply default
```

First-run leftovers: [Getting started](../getting-started/). Command list: [Automate](../automate/).
