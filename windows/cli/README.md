# Windows WSL Setup

Single Windows binary. Users download `windows-wsl-setup.exe` from GitHub Releases. They do not clone the repo or run scripts.

```
windows-wsl-setup              Collect or Restore
windows-wsl-setup collect      scan this PC → winget manifest + kit on a data drive
windows-wsl-setup restore      pick packages from the kit → winget install; remount Dev Drive / WSL; Brave bookmarks + extensions.html
windows-wsl-setup inventory    JSON scan
```

Collect writes `apps/winget-selected.json` (winget import schema) plus `inventory/apps.json` (ids and names). Restore shows every package in that manifest; the user ticks what to install.

Build (maintainers):

```
cargo build --release --manifest-path windows/cli/Cargo.toml
```
