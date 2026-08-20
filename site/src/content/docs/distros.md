---
title: Distros
description: Ubuntu, Debian, and Arch.
order: 7
group: Reference
---

New WSL provisions **Ubuntu** (default), **Debian**, or **Arch**. Restore remounts existing WSL instances. Distros already on the machine stay installed.

<div class="table-wrap">
<table>
<thead>
<tr><th>Distro</th><th>Name</th><th>Bootstrap</th></tr>
</thead>
<tbody>
<tr><td>Ubuntu</td><td>Ubuntu-26.04</td><td>apt</td></tr>
<tr><td>Debian</td><td>Debian</td><td>apt</td></tr>
<tr><td>Arch</td><td>archlinux</td><td>pacman</td></tr>
</tbody>
</table>
</div>

CLIs come from Homebrew. The distro package manager is bootstrap only.

Fedora, Kali, and openSUSE are restore-only. Requires x86_64.

```
wwm distros
```
