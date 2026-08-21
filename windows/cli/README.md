# wwm (Windows WSL Manager)

People download `wwm.exe` from Releases, or:

```
irm https://pjmagee.github.io/wwm/install.txt | iex
```

They never clone this folder.

```
wwm
wwm collect
wwm restore
wwm new-wsl
wwm new-wsl --profile home --distro Debian
wwm new-wsl --profile blank --distro fedora --location D:\WSL\Fedora
wwm distros
wwm distro move Debian D:\WSL\Debian
wwm distro clone Ubuntu-26.04 Ubuntu-dev --location D:\WSL\Ubuntu-dev
wwm profiles
wwm suggest
wwm apply default --distro Ubuntu-26.04
```
