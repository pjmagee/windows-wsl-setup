---
title: Agents
description: Install the skill so Grok, Claude, Codex, or Copilot can drive Collect, Restore, and Apply.
order: 3
group: Start
---

The skill teaches an agent to download the exe and run Collect, Restore, New WSL, or Apply. Install it at **user** scope so it works on a used PC and on a fresh Windows 11.

## Install the skill

The skill id is `wwm-cli`. Pick **one** installer.

**GitHub CLI** 2.90+ — `--agent` is required; default is Copilot, not Grok:

```
gh skill install pjmagee/wwm wwm-cli --scope user --agent grok
gh skill install pjmagee/wwm wwm-cli --scope user --agent claude-code
gh skill install pjmagee/wwm wwm-cli --scope user --agent github-copilot
gh skill install pjmagee/wwm wwm-cli --scope user --agent codex
```

**skills.sh** (any detected agent; needs Node):

```
npx skills add pjmagee/wwm --skill wwm-cli -g -y
```

**Grok plugin marketplace:**

```
grok plugin marketplace add pjmagee/wwm
grok plugin install wwm-cli --trust
```

**Claude Code plugin marketplace:**

```
/plugin marketplace add pjmagee/wwm
/plugin install wwm-cli@wwm
```

Or ask the agent to read [SKILL.md](https://raw.githubusercontent.com/pjmagee/wwm/main/skills/wwm-cli/SKILL.md).

## Drive

1. Confirm Windows (not WSL, not Git Bash).
2. Install `wwm.exe` into `~\.wwm` with the PowerShell block above.
3. **Collect** on the used PC. **Restore** on the new PC. **New WSL** or **apply** if there is no kit. Linux profiles: `blank` (sudo only), `home`, `work`.
4. Password manager first, then Brave.
5. Do not format data drives. Do not unregister a distro unless the human confirms (`wwm distro remove <name> --yes`).

```
wwm spec
wwm suggest
wwm map <winget-id>
wwm profile list
wwm new-wsl --profile blank --distro Debian
wwm apply default
```

Leftovers: [Getting started](../getting-started/). Command list: [Automate](../automate/).
