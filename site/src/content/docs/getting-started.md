---
title: Getting started
description: "Four jobs: kit a PC, back up WSL, manage profiles, or let an agent drive the CLI."
order: 1
group: Start
---

`wwm.exe` runs on **Windows 11**. It is not a Linux tool.

| You want | Open |
|---|---|
| Snapshot this PC and reinstall Windows 11 | [Collect a kit](#1-collect-a-kit-for-a-fresh-windows-pc) |
| Keep or bring back a Linux disk | [Backup and restore WSL](#2-backup-and-restore-wsl) |
| Empty official distro, or one with packages | [New WSL](../wsl/) |
| Named Windows and Linux software lists | [Profiles](#3-profiles-for-windows-and-wsl) |
| An agent that can drive the CLI | [Agents](#4-agents) |

```
wwm
wwm collect
wwm restore
wwm new-wsl --profile blank --distro Debian
wwm new-wsl --profile home --distro Ubuntu-26.04
wwm profiles
wwm spec
```

## 1. Collect a kit for a fresh Windows PC

On the used PC: **Collect**. Destination must not be `C:`. Tick winget apps and anything else you want in the kit (WSL disks, Dev Drive, Docker data, Brave).

On the new PC: install Windows 11, leave data drives alone, **Restore**. Winget installs in catalog order (password manager, then your browser, then the rest).

Details: [Kits](../kits/).

## 2. Backup and restore WSL

Collect can copy each ticked distro’s VHDX into the kit. Restore **imports** that disk. Other distros already on the machine stay installed.

There is no disk to bring back? Use **New WSL**:

- `blank` — distro + Linux user + passwordless sudo. No extra packages.
- `home` / `work` — the same host steps, then Linux CLIs from that profile.

```
wwm new-wsl --profile blank --distro Debian
wwm new-wsl --profile home --distro Ubuntu-26.04
wwm new-wsl --profile blank --distro fedora --location D:\WSL\Fedora
```

Details: [WSL](../wsl/), [Distros](../distros/).

## 3. Profiles for Windows and WSL

A **bundle** names a Windows list and a Linux list. Shipped: `blank`, `default`, `home`, `work`. Tick packages, save (`s`), suggest from this PC (`g`).

```
wwm profile list
wwm profile new media-pc --from home --name "Media PC"
wwm profile add media-pc --linux kubectl --windows Valve.Steam
wwm apply media-pc
```

Apply installs Windows apps with winget. It can also create a distro. It does not remount a VHDX and does not uninstall Windows apps.

Details: [Profiles](../profiles/).

## 4. Agents

Install the skill, then `wwm spec` for the OpenCLI map. JSON on stdout, errors on stderr.

Details: [Agents](../agents/), [Automate](../automate/).

## After Restore or New WSL

Clicks the exe cannot do: 1Password SSH agent, Brave **Add**, game library folder, Docker WSL integration for that distro, VS Code WSL extension.

<p class="note">Do not format data drives. Do not <code>wsl --unregister</code> unless you intend to delete that disk (<code>wwm distro remove NAME --yes</code>).</p>
