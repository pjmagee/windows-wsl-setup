---
title: Download
description: wwm.exe for Windows 11 x86_64.
order: 2
group: Start
---

Download `wwm.exe` from [Releases](https://github.com/pjmagee/wwm/releases). Put it in `~\.wwm` on **Windows** (that is the install folder for the exe, not a Linux path).

```
New-Item $HOME\.wwm -ItemType Directory -Force | Out-Null
Invoke-WebRequest -UseBasicParsing https://github.com/pjmagee/wwm/releases/latest/download/wwm.exe -OutFile $HOME\.wwm\wwm.exe
$env:Path = "$HOME\.wwm;$env:Path"
wwm
```

```
wwm collect
wwm restore
wwm new-wsl --profile blank --distro Debian
wwm profiles
wwm spec
```

[Getting started](../getting-started/). JSON CLI: [Automate](../automate/).
