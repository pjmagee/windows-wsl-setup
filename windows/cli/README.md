# Windows WSL Setup (binary)

People download `windows-wsl-setup.exe` from Releases. They never clone this folder.

```
windows-wsl-setup
windows-wsl-setup collect
windows-wsl-setup restore
windows-wsl-setup new-wsl
windows-wsl-setup new-wsl --profile home --distro Debian
windows-wsl-setup distros
windows-wsl-setup profiles
windows-wsl-setup suggest
windows-wsl-setup apply default --distro Ubuntu-26.04
```

Maintainers:

```
cargo build --release --manifest-path windows/cli/Cargo.toml
```

Explorer icon: [`assets/app.ico`](assets/app.ico). Regenerate with `python assets/gen_icon.py` (Pillow).

Catalogs and shipped profiles are `include_str!` from `profiles/` at compile time.
