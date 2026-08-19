#Requires -Version 5.1
<#
.SYNOPSIS
  Write a Windows WSL Setup kit from a capture selection.

  v1 copies small files (dotfiles, Brave, winget JSON, generated AGENTS.md).
  VHDX / WSL export is recorded in KIT.json but not copied unless -IncludeVhd
  (elevated; later PR).
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $KitRoot,
    [string] $SelectionJson,
    [string] $SelectionPath,
    [string] $RepoRoot,
    [switch] $IncludeVhd
)

Set-StrictMode -Version 1
$ErrorActionPreference = 'Stop'

if (-not $RepoRoot) {
    $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

if ($SelectionPath) {
    $SelectionJson = Get-Content -LiteralPath $SelectionPath -Raw
}
if ([string]::IsNullOrWhiteSpace($SelectionJson)) { throw 'SelectionJson or SelectionPath required' }

$sel = $SelectionJson | ConvertFrom-Json
if (-not $sel.kitRoot) { $sel | Add-Member -NotePropertyName kitRoot -NotePropertyValue $KitRoot }
$KitRoot = $sel.kitRoot
if ([string]::IsNullOrWhiteSpace($KitRoot)) { throw 'kitRoot missing' }
if ($KitRoot -match '^[cC]:') { throw 'Refusing to write a kit on C:. Pick a data drive.' }

function New-Dir([string] $p) {
    if (-not (Test-Path -LiteralPath $p)) { New-Item -ItemType Directory -Force -Path $p | Out-Null }
}

function Copy-Safe([string] $From, [string] $To) {
    if (-not (Test-Path -LiteralPath $From)) { return $false }
    $dir = Split-Path -Parent $To
    New-Dir $dir
    Copy-Item -LiteralPath $From -Destination $To -Force
    return $true
}

New-Dir $KitRoot
foreach ($s in @('inventory', 'config', 'config\ssh', 'config\git', 'config\grok', 'config\terminal', 'config\powershell', 'browser', 'apps', 'vhdx')) {
    New-Dir (Join-Path $KitRoot $s)
}

$status = [ordered]@{
    StartedAt     = (Get-Date).ToString('o')
    KitRoot       = $KitRoot
    IncludeVhd    = [bool]$IncludeVhd
    Dotfiles      = @()
    Brave         = $false
    WingetExport  = $false
    VhdCopied     = $false
    Errors        = @()
}

$up = $env:USERPROFILE
if ($sel.dotfiles) {
    $map = @(
        @{ From = Join-Path $up '.wslconfig'; To = Join-Path $KitRoot 'config\wslconfig' }
        @{ From = Join-Path $up '.gitconfig'; To = Join-Path $KitRoot 'config\git\gitconfig' }
        @{ From = Join-Path $up '.ssh\config'; To = Join-Path $KitRoot 'config\ssh\config' }
        @{ From = Join-Path $up '.ssh\known_hosts'; To = Join-Path $KitRoot 'config\ssh\known_hosts' }
        @{ From = Join-Path $up '.grok\config.toml'; To = Join-Path $KitRoot 'config\grok\config.toml' }
        @{ From = Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json'; To = Join-Path $KitRoot 'config\terminal\settings.json' }
        @{ From = Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'PowerShell\Microsoft.PowerShell_profile.ps1'; To = Join-Path $KitRoot 'config\powershell\Microsoft.PowerShell_profile.ps1' }
    )
    foreach ($m in $map) {
        if (Copy-Safe $m.From $m.To) { $status.Dotfiles += $m.To }
    }
}

if ($sel.browser) {
    $brave = Join-Path $env:LOCALAPPDATA 'BraveSoftware\Brave-Browser\User Data\Default'
    Copy-Safe (Join-Path $brave 'Bookmarks') (Join-Path $KitRoot 'browser\Bookmarks') | Out-Null
    Copy-Safe (Join-Path $brave 'Bookmarks.bak') (Join-Path $KitRoot 'browser\Bookmarks.bak') | Out-Null
    $exts = @()
    if ($sel.braveExtensions) { $exts = @($sel.braveExtensions) }
    $html = New-Object System.Text.StringBuilder
    [void]$html.AppendLine('<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Brave extensions</title>')
    [void]$html.AppendLine('<style>body{font-family:Segoe UI,sans-serif;max-width:720px;margin:40px auto;color:#e7eef7;background:#11161c}a{color:#6ee7b7}ol{line-height:1.8}.note{background:#1b222b;padding:12px 16px;border-radius:8px}</style></head><body>')
    [void]$html.AppendLine('<h1>Install these Brave extensions</h1>')
    [void]$html.AppendLine('<p class="note">Click <b>Add to Brave</b> on each Chrome Web Store page. Bookmarks from the kit are restored separately.</p><ol>')
    foreach ($e in $exts) {
        $id = [string]$e.id
        $name = [string]$e.name
        if (-not $name) { $name = $id }
        [void]$html.AppendLine(('  <li><a href="https://chromewebstore.google.com/detail/{0}">{1}</a></li>' -f $id, ($name -replace '<', '')))
    }
    [void]$html.AppendLine('</ol></body></html>')
    $htmlPath = Join-Path $KitRoot 'browser\extensions.html'
    [System.IO.File]::WriteAllText($htmlPath, $html.ToString())
    $exts | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $KitRoot 'inventory\brave-extensions.json') -Encoding UTF8
    $status.Brave = $true
}

$apps = @()
if ($sel.apps) { $apps = @($sel.apps) }
$selectedDoc = @{
    '$schema' = 'https://aka.ms/winget-packages.schema.2.0.json'
    Sources   = @(@{
        SourceDetails = @{ Name = 'winget' }
        Packages      = @($apps | ForEach-Object { @{ PackageIdentifier = $_ } })
    })
}
$selectedDoc | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $KitRoot 'apps\winget-selected.json') -Encoding UTF8
try {
    & winget.exe export --output (Join-Path $KitRoot 'apps\winget-export-raw.json') --accept-source-agreements --disable-interactivity 2>$null | Out-Null
    $status.WingetExport = Test-Path (Join-Path $KitRoot 'apps\winget-export-raw.json')
} catch {
    $status.Errors += $_.Exception.Message
}

if ($IncludeVhd) {
    $status.Errors += 'VHDX copy is not in this Backup-Kit version. Re-run with the elevated Dev Drive script (next).'
}

$profile = 'home'
if ($sel.linuxProfile) { $profile = [string]$sel.linuxProfile }
if ($profile -eq 'universal') { $profile = 'home' }
if ($sel.linuxTools) {
    $sel.linuxTools | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $KitRoot 'inventory\linux-tools.json') -Encoding UTF8
}

$kit = [ordered]@{
    schemaVersion      = 1
    createdAt          = (Get-Date).ToString('o')
    computer           = $env:COMPUTERNAME
    user               = $env:USERNAME
    windowsUserProfile = $up
    linuxProfile       = $profile
    linuxTools         = $sel.linuxTools
    kitRoot            = $KitRoot
    repo               = 'https://github.com/pjmagee/windows-wsl-setup.git'
    selections         = @{
        apps       = $apps
        wsl        = @($sel.wsl)
        devDrive   = [bool]$sel.devDrive
        dockerData = [bool]$sel.dockerData
        browser    = [bool]$sel.browser
        dotfiles   = [bool]$sel.dotfiles
        includeVhd = [bool]$IncludeVhd
    }
}
$kit | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $KitRoot 'KIT.json') -Encoding UTF8
$status | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $KitRoot 'inventory\BACKUP-STATUS.json') -Encoding UTF8

$wslList = 'none'
if ($sel.wsl) { $wslList = (@($sel.wsl) -join ', ') }
$agents = @"
# Restore this machine (generated kit)

If you are an agent on a freshly reset Windows 11 PC and the human says
anything like **restore this machine** / **restore Dev Drive & WSL**,
**execute this file**. Do not invent a second plan. Do not format data
drives. Do not ``wsl --unregister`` unless the human confirms.

- Kit: ``$KitRoot``
- Old computer: $($env:COMPUTERNAME)
- Windows user: $($env:USERNAME)
- Linux profile after WSL is up: ``./install.sh $profile`` (inside Ubuntu-26.04)
- Repo: https://github.com/pjmagee/windows-wsl-setup.git

## Order

1. Clone the repo if needed (HTTPS is fine).
2. ``windows\host\Restore-Kit.ps1 -KitRoot '$KitRoot'``
   (letters by volume GUID/label, Dev Drive VHDX, WSL import-in-place,
   host dotfiles, winget one-by-one from ``apps\winget-selected.json``,
   Brave bookmarks + ``browser\extensions.html``).
3. ``windows\bootstrap.ps1 -SkipLinuxInstall``
4. Copy ``inventory/linux-tools.json`` to ``~/.config/wsl-setup/tools.json`` if present
   (home/work ticks from capture).
5. Inside Ubuntu-26.04: ``cd ~/code/windows-wsl-setup && git pull && ./install.sh $profile``
   Do **not** run ``install.sh work`` on a home kit. There is no universal profile.

## Selected

- Apps: $($apps.Count) winget ids
- WSL: $wslList
- Dev Drive: $($sel.devDrive)
- Docker data VHDX: $($sel.dockerData)
- Brave: $($sel.browser)
- Dotfiles: $($sel.dotfiles)

## Manual leftovers

- 1Password → Settings → Developer → Use the SSH agent
- Windows OpenSSH Authentication Agent service **off**
- Steam library path if games live on another drive
- Docker Desktop → WSL integration → Ubuntu-26.04
- NVIDIA App if it was not in winget

## Invariants

- Linux toolchains stay inside WSL. Do not winget-install Node/Go/Rust/Python to "fix" a missing CLI.
- After WSL import-in-place, grant ``NT VIRTUAL MACHINE\Virtual Machines`` Full control on ``ext4.vhdx`` (old VM SID causes Access Denied).
- ``winget import`` of the raw export can abort (Battle.net ``--location``). Install selected ids one-by-one.
"@
Set-Content -LiteralPath (Join-Path $KitRoot 'AGENTS.md') -Value $agents -Encoding UTF8

$start = @"
Windows WSL Setup kit — $($env:COMPUTERNAME) $(Get-Date -Format yyyy-MM-dd)

This folder is the backup. On the new PC:

  1. Sign into the same Microsoft account. Do not clean data drives.
  2. Download windows-wsl-setup.exe from GitHub Releases (or use the copy in this kit).
  3. Tell the agent:  Read $KitRoot\AGENTS.md and restore this machine.

Small files are in this kit. VHDX copy (Dev Drive / WSL) may still need
an elevated backup pass if it was not included.
"@
Set-Content -LiteralPath (Join-Path $KitRoot 'START-HERE.txt') -Value $start -Encoding UTF8

Write-Output ($status | ConvertTo-Json -Depth 6)
