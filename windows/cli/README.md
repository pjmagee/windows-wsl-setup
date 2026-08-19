# Windows WSL Setup (binary)

People download `windows-wsl-setup.exe` from Releases. They never clone this folder.

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup profiles
windows-wsl-setup suggest
windows-wsl-setup apply default
```

Maintainers:

```
cargo build --release --manifest-path windows/cli/Cargo.toml
```

Catalogs and shipped profiles are `include_str!` from `profiles/` at compile time.
