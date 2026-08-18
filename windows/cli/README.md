# wsl-setup (Windows TUI)

Native console app. No browser.

```powershell
cargo build --release --manifest-path windows\cli\Cargo.toml
.\windows\cli\target\release\wsl-setup.exe capture
```

Or from the repo root, `windows\host\capture.ps1` launches this binary (builds it if needed).

| Command | What |
|---|---|
| `wsl-setup` / `capture` | TUI: dest, home/work, Linux extras, WSL, host, winget, write kit |
| `wsl-setup inventory` | JSON scan to stdout |

Keys: `tab` section, `j`/`k` move, `space` toggle, `w` work-tick on Linux extras, `/` filter apps, `W` or Enter on Write to emit the kit, `q` quit.
