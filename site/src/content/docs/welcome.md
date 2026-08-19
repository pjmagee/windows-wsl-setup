---
title: Welcome
description: What problem we solve, and which path to take.
order: 1
group: Start
---

You have a **messy Windows 11 PC**. Maybe you used WSL before, maybe not. You are about to reset, or you just installed Windows 11 again.

**Windows WSL Setup** exports what matters, then puts it back — or builds a new Linux environment from a profile. One Windows exe. Optional: let **Grok, Claude, or Codex** run it for you.

| You have | You want | Do this |
|---|---|---|
| A used PC | A snapshot that survives a reinstall | **Collect** |
| That snapshot on a data drive | The same apps and disks on a new Windows 11 | **Restore** |
| No snapshot | A new Linux environment + a software list | **New WSL** or **Apply a profile** |
| An agent | The agent to drive the exe | [Install the skill](../agents/) |

A **kit** is a snapshot of *this* machine. A **profile** is a named list of *what you want*. Restore wins when you still have the Linux disk. Apply does not remount disks.

## Install order (why it feels usable fast)

On a fresh PC we install in this order:

1. **Password manager** — unlocks vaults and SSH.
2. **Your browser** — logins, bookmarks, extension list.
3. **Daily desktop** — Terminal, archive tool, PowerToys.
4. Everything else (editors, Git, Linux, Docker, games).

Linux CLIs inside WSL follow a similar idea: shell first, then runtimes, then cloud tools, then coding agents.
