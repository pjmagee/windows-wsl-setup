---
title: Agents
description: Install our skill so Grok, Claude, Codex, or Copilot can drive the CLI.
order: 3
group: Start
---

The product is **agent-ready**. The skill teaches an agent to download the exe, understand every command, collect a snapshot, restore disks, and apply a profile.

## Install the skill

From any machine with Node:

```
npx skills add pjmagee/windows-wsl-setup
```

That is the [skills CLI](https://skills.sh/) (works with Claude Code, Codex, Copilot, Cursor, OpenCode, and others). Global install:

```
npx skills add pjmagee/windows-wsl-setup -g -y
```

Grok already looks at `.agents/skills/` in this repo. After `npx skills add`, other agents get the same `SKILL.md`.

## What the agent should do

1. Download `windows-wsl-setup.exe` from GitHub Releases (do not clone unless you are changing the product).
2. **Collect** on the used PC (kit on a non-system drive).
3. **Restore** on the new PC, or **New WSL** / **apply** if there is no kit.
4. Install **password manager first**, then **the browser**, then the rest.
5. Never format data drives. Never unregister a Linux distro unless the human confirms.

JSON CLI (always JSON on stdout):

```
windows-wsl-setup suggest
windows-wsl-setup map <winget-id>
windows-wsl-setup profile list
windows-wsl-setup apply default
```

Human leftovers the exe cannot click: password-manager SSH agent, game library folders, Docker’s Linux integration checkbox, **Add** on the browser extensions page.
