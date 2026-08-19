# Windows WSL Setup (host helpers)

Capture a used Windows 11 PC into a **kit** on a non-`C:` drive. Restore is agent-driven from that kit's `AGENTS.md`.

Linux toolchain install is still [`install.sh`](../../install.sh) inside Ubuntu 26.04. This folder does not put Node/Go/Rust/Python on Windows.

## Capture

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\windows\host\capture.ps1
```

- Launches the Rust TUI in `windows/cli` (builds a release exe if needed). No browser.
- `Inventory.ps1` — winget export + list, volume GUIDs, HKCU Lxss distros, Dev Drive VHDX, Docker data VHDX, Brave Default, dotfiles.
- Linux extras: tick **home** and/or **work** (base tools are always on). Saved as `inventory/linux-tools.json`. No `universal` profile.
- **Write kit** runs `Backup-Kit.ps1` (small files + generated playbook). VHDX copy is not in this pass.

Refuses `C:` as the kit root.

## Restore (later scripts)

Kit `AGENTS.md` is the contract. Intended order: drive letters by GUID/label → Dev Drive VHDX + WSL import-in-place (then ACL `S-1-5-83-0` on `ext4.vhdx`) → dotfiles → `bootstrap.ps1 -SkipLinuxInstall` → one-by-one winget from `apps\winget-selected.json` → Brave `browser\extensions.html` → `./install.sh <linuxProfile>` inside Ubuntu.

## Files

| File | Role |
|---|---|
| `capture.ps1` | Localhost UI |
| `Inventory.ps1` | Scan (also `powershell -File Inventory.ps1` prints JSON) |
| `Backup-Kit.ps1` | Write kit from the UI payload |
| `ui/` | HTML/JS/CSS |
