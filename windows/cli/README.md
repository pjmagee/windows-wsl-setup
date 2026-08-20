# wwm (Windows WSL Manager)

People download `wwm.exe` from Releases, or:

```
curl.exe -L --create-dirs -o $HOME\.wwm\wwm.exe https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/wwm.exe; $env:Path = "$HOME\.wwm;$env:Path"; wwm
```

They never clone this folder.

```
wwm
wwm collect
wwm restore
wwm new-wsl
wwm new-wsl --profile home --distro Debian
wwm distros
wwm profiles
wwm suggest
wwm apply default --distro Ubuntu-26.04
```
