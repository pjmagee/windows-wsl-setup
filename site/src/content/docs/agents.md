---
title: Agents
description: Skill plus JSON CLI. wwm spec is the OpenCLI map.
order: 3
group: Start
---

`wwm.exe` is agent-first: JSON on stdout, errors on stderr, OpenCLI at `wwm spec`.

Install the skill at **user** scope. Host is Windows (not WSL, not Git Bash).

The skill id is `wwm-cli`. Pick **one** installer.

**GitHub CLI** 2.90+ (`--agent` is required):

```
gh skill install pjmagee/wwm wwm-cli --scope user --agent grok
gh skill install pjmagee/wwm wwm-cli --scope user --agent claude-code
gh skill install pjmagee/wwm wwm-cli --scope user --agent github-copilot
gh skill install pjmagee/wwm wwm-cli --scope user --agent codex
```

**skills.sh:**

```
npx skills add pjmagee/wwm --skill wwm-cli -g -y
```

**Grok:**

```
grok plugin marketplace add pjmagee/wwm
grok plugin install wwm-cli --trust
```

**Claude Code:**

```
/plugin marketplace add pjmagee/wwm
/plugin install wwm-cli@wwm
```

Then:

```
wwm spec
```

```
wwm collect
wwm restore
wwm new-wsl --profile blank --distro Debian
wwm apply home --windows-only
```

Do not format data drives. Do not unregister a distro unless you intend to delete that disk (`wwm distro remove NAME --yes`).

Command list: [Automate](../automate/).
