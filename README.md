# Windows WSL Setup

One Windows app. Download `windows-wsl-setup.exe` from this repo’s **Releases**.

You do not clone the repo. You do not run scripts.

## Before you reset Windows

1. Put your files on a drive that is **not C:** (games, photos, this kit).
2. Run `windows-wsl-setup.exe` and choose **Collect**.
3. Pick that data drive. Tick the winget apps you want later, plus WSL and the Dev Drive if you use them.
4. Write the kit. Keep that folder. Resetting C: must not delete it.

## After a fresh Windows 11 install

1. Sign in. Do **not** wipe the data drives.
2. Run the same exe (from Releases, or the copy inside the kit folder).
3. Choose **Restore**. Pick the kit. Tick the apps to install.
4. Apply. The app:
   - installs the ticked apps with winget
   - remounts the Dev Drive
   - imports your WSL distro
   - copies Brave bookmarks
   - opens a page of Chrome Web Store links for extensions (click **Add to Brave**)

```
windows-wsl-setup           Collect or Restore
windows-wsl-setup collect
windows-wsl-setup restore
```

## What you still do by hand

The app cannot log into other apps or click browser-store buttons.

- 1Password: Settings → Developer → **Use the SSH agent**
- Steam: add your existing library folder
- Docker Desktop: enable WSL integration for your distro
- Brave extensions: the page it opens
- NVIDIA App if it was not on winget

## Linux tools (optional, after WSL is back)

The Windows app does not install Node, Git, or language runtimes on Windows. Those live **inside Ubuntu**.

Once Ubuntu opens, from a Linux terminal:

```bash
cd ~/code/windows-wsl-setup && ./install.sh home    # or: ./install.sh work
```

Details for that half: [AGENTS.md](AGENTS.md).
