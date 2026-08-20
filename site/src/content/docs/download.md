---
title: Download
description: Collect a kit. Restore Windows 11 and WSL.
order: 2
group: Start
---

<p class="cta-row">
  <a class="btn" href="https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/wwm.exe">Download for Windows</a>
  <a class="btn ghost" href="https://github.com/pjmagee/windows-wsl-manager/releases">View releases</a>
</p>

<p class="cta-meta">wwm.exe · Windows 11 · x86_64 · installs to ~\.wwm</p>

PowerShell — download, put it on PATH for this session, run:

```
curl.exe -L --create-dirs -o $HOME\.wwm\wwm.exe https://github.com/pjmagee/windows-wsl-manager/releases/latest/download/wwm.exe; $env:Path = "$HOME\.wwm;$env:Path"; wwm
```

Then:

```
wwm collect
wwm restore
wwm new-wsl
wwm profiles
```

Which mode: [Getting started](../getting-started/). JSON CLI: [Automate](../automate/).

If Windows just enabled WSL, reboot when asked and run `wwm` again.
