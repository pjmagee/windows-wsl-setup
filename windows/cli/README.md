# Windows WSL Setup (binary)

People download `windows-wsl-setup.exe` from Releases. They never clone this folder.

```
windows-wsl-setup           Collect or Restore
windows-wsl-setup collect
windows-wsl-setup restore
```

Maintainers:

```
cargo build --release --manifest-path windows/cli/Cargo.toml
```
