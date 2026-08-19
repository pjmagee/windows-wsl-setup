# Windows WSL Setup

Download `windows-wsl-setup.exe` from **Releases**. Do not clone this repo.

A **kit** is a snapshot of this PC (Collect → Restore). A **profile** is a named list of software you want (shipped `home` / `work` / `default`, or your own). Use a profile when you have nothing to restore.

## 1. Old PC — Collect

Run the exe. Choose **Collect**.

- Pick a drive that is **not C:** (the kit must survive a Windows reset).
- Tick the apps you want later (grouped by category).
- Tick your WSL distro and Dev Drive if you use them.
- Write the kit. Keep that folder.

## 2. New PC — Restore

Install Windows 11. Do **not** wipe the data drives.

Run the same exe. Choose **Restore**.

- Pick the kit.
- Tick the apps to install.
- Apply.

The app installs those apps with winget, remounts the Dev Drive, brings WSL back, copies Brave bookmarks, and opens a page of extension links. Click **Add to Brave** on that page.

Your WSL disk is the one you already had. Linux tools come back with it.

## 3. New WSL (empty Ubuntu, not a restore)

Choose **New WSL** when there is no Linux disk to bring back.

Always **Ubuntu 26.04**. We do not offer Fedora, Arch, or “pick a package manager”. apt for system packages. Homebrew for CLIs.

Pick a **linux profile** (`home`, `work`, or a custom one). Enter. The app installs Ubuntu, makes a passwordless sudo user from your Windows username, then installs that profile.

## 4. Profiles (no kit)

Choose **Profiles** on a blank machine, or to build a custom set.

- Shipped bundles: **default** (fresh PC), **home**, **work**.
- Tick Windows (winget) and Linux tools by category.
- **g** suggests a profile from the software already on this PC. SDKs and cloud CLIs prefer the Linux catalog (Node/Azure/Go on Windows → uv/azure-cli/go in WSL).
- **s** saves `custom` under `%USERPROFILE%\.windows-wsl-setup\profiles\`.
- **Apply** installs winget packages and can create Ubuntu 26.04. It does **not** remount disks.

Agents drive the same thing from the CLI (JSON on stdout):

```
windows-wsl-setup suggest
windows-wsl-setup map Microsoft.AzureCLI
windows-wsl-setup search linux kubectl
windows-wsl-setup search winget terraform
windows-wsl-setup profile new my-dev --from home
windows-wsl-setup profile add my-dev --linux kubectl --windows Brave.Brave
windows-wsl-setup apply my-dev
```

## 5. A few clicks the app cannot make

- 1Password → Settings → Developer → Use the SSH agent
- Steam → add your existing library folder
- Docker Desktop → WSL integration for your distro

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
```
