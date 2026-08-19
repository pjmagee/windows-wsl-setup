# Windows WSL Setup

Download `windows-wsl-setup.exe` from **Releases**. Do not clone this repo.

## 1. Old PC — Collect

Run the exe. Choose **Collect**.

- Pick a drive that is **not C:** (the kit must survive a Windows reset).
- Tick the apps you want later (Steam, Brave, Docker, …).
- Tick your WSL distro and Dev Drive if you use them.
- Write the kit. Keep that folder.

## 2. New PC — Restore

Install Windows 11. Do **not** wipe the data drives.

Run the same exe (from Releases, or the copy in the kit folder). Choose **Restore**.

- Pick the kit.
- Tick the apps to install.
- Apply.

The app installs those apps with winget, remounts the Dev Drive, brings WSL back, copies Brave bookmarks, and opens a page of extension links. Click **Add to Brave** on that page.

Your WSL disk is the one you already had. Linux tools come back with it. Nobody runs install scripts.

## 3. A few clicks the app cannot make

- 1Password → Settings → Developer → Use the SSH agent
- Steam → add your existing library folder
- Docker Desktop → WSL integration for your distro

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
```
