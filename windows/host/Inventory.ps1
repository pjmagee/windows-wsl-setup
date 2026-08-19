#Requires -Version 5.1
<#
.SYNOPSIS
  Scan this Windows 11 machine for a Windows WSL Setup capture kit.

.DESCRIPTION
  No admin required. Does not copy anything. Emits a JSON-ready object:
  volumes (non-C: destinations), winget packages, WSL distros + VHDX sizes,
  Dev Drive, Docker data VHDX, Brave Default extensions/bookmarks, dotfiles.

  PowerShell 5.1 compatible (work laptops).
#>
[CmdletBinding()]
param(
    [switch] $AsJson
)

Set-StrictMode -Version 1
$ErrorActionPreference = 'Continue'

function Get-FileSizeBytes {
    param([string] $Path)
    if (-not $Path -or -not (Test-Path -LiteralPath $Path)) { return 0 }
    return [int64](Get-Item -LiteralPath $Path).Length
}

function ConvertTo-UncPath {
    param([string] $Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $Path }
    return ($Path -replace '^\\\\\?\\', '')
}

function Test-DefaultKeepApp {
    param([string] $Id)
    $keep = @(
        '^Valve\.Steam$', '^Brave\.', '^AgileBits\.1Password', '^Docker\.DockerDesktop$',
        '^Git\.Git$', '^Microsoft\.VisualStudioCode$', '^Anysphere\.Cursor$',
        '^Discord\.', '^Microsoft\.Office$', '^Blizzard\.', '^EpicGames\.',
        '^ElectronicArts\.', '^Ubisoft\.', '^GOG\.', '^Corsair\.', '^SteelSeries\.',
        '^NordSecurity\.', '^Tailscale\.', '^WinSCP\.', '^7zip\.', '^VideoLAN\.',
        '^Plex\.', '^Microsoft\.WindowsTerminal$', '^JanDeDobbeleer\.OhMyPosh$',
        '^Microsoft\.PowerToys$', '^Microsoft\.PowerShell$', '^NexusMods\.',
        '^Rockstar', '^Paradox', '^CloudImperium', '^Yubico\.', '^Anthropic\.Claude',
        '^BitSum\.ProcessLasso$', '^Microsoft\.VisualStudio\.BuildTools$',
        '^xAI\.GrokBuild$', '^Devolutions\.UniGetUI$'
    )
    $skip = @(
        '^Python\.Python', '^Python\.Launcher', '^Rustlang\.', '^Microsoft\.OpenJDK',
        '^Microsoft\.DotNet\.SDK', '^Microsoft\.Azure', '^OpenJS\.NodeJS',
        '^Microsoft\.VCRedist', '^Microsoft\.UI\.Xaml', '^Microsoft\.WindowsAppRuntime',
        '^Microsoft\.VCLibs', '^Microsoft\.Edge', '^Microsoft\.OneDrive',
        '^Microsoft\.AppInstaller', '^Microsoft\.WSL$', '^Microsoft\.Teams',
        '^Microsoft\.WindowsApp$', '^Microsoft\.DotNet\.Native'
    )
    foreach ($re in $skip) { if ($Id -match $re) { return $false } }
    foreach ($re in $keep) { if ($Id -match $re) { return $true } }
    return $true
}

function Get-WingetPackages {
    $packages = @()
    $tmp = Join-Path $env:TEMP ("wsl-setup-winget-{0}.json" -f [guid]::NewGuid().ToString('n'))
    try {
        & winget.exe export --output $tmp --accept-source-agreements --disable-interactivity 2>$null | Out-Null
        if (Test-Path -LiteralPath $tmp) {
            $raw = Get-Content -LiteralPath $tmp -Raw -ErrorAction SilentlyContinue
            if ($raw) {
                $exp = $raw | ConvertFrom-Json
                foreach ($src in @($exp.Sources)) {
                    foreach ($p in @($src.Packages)) {
                        $id = [string]$p.PackageIdentifier
                        if (-not $id) { continue }
                        $packages += [pscustomobject]@{
                            id      = $id
                            version = [string]$p.Version
                            keep    = [bool](Test-DefaultKeepApp $id)
                        }
                    }
                }
            }
        }
    } catch { }
    finally {
        if (Test-Path -LiteralPath $tmp) { Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
    }

    $byId = @{}
    foreach ($p in $packages) { $byId[$p.id] = $p }

    # Names from `winget list` (export has ids only).
    try {
        $lines = @(cmd /c "winget list --accept-source-agreements --disable-interactivity 2>nul")
        $header = $null
        foreach ($line in $lines) {
            if ($line -match '^Name\s+Id\s+Version') { $header = $line; break }
        }
        if ($header) {
            $idAt = $header.IndexOf('Id')
            $verAt = $header.IndexOf('Version')
            if ($idAt -gt 0 -and $verAt -gt $idAt) {
                foreach ($line in $lines) {
                    if ($line.Length -lt $verAt) { continue }
                    if ($line -match '^Name\s+Id' -or $line -match '^-+' -or $line -match 'upgrades available') { continue }
                    $name = $line.Substring(0, $idAt).Trim()
                    $id = $line.Substring($idAt, $verAt - $idAt).Trim()
                    if (-not $id -or $id -match '\s') { continue }
                    if ($byId.ContainsKey($id)) {
                        $obj = $byId[$id]
                        $byId[$id] = [pscustomobject]@{
                            id      = $id
                            name    = $name
                            version = $obj.version
                            keep    = $obj.keep
                        }
                    } elseif ($id -match '^[A-Za-z0-9][A-Za-z0-9_.-]+\.[A-Za-z0-9][A-Za-z0-9_.-]+') {
                        $byId[$id] = [pscustomobject]@{
                            id      = $id
                            name    = $name
                            version = ''
                            keep    = [bool](Test-DefaultKeepApp $id)
                        }
                    }
                }
            }
        }
    } catch { }

    $out = @()
    foreach ($k in ($byId.Keys | Sort-Object)) {
        $p = $byId[$k]
        if (-not $p.PSObject.Properties['name']) {
            $p = [pscustomobject]@{ id = $p.id; name = $p.id; version = $p.version; keep = $p.keep }
        }
        $out += $p
    }
    return $out
}

function Get-WslDistros {
    $distros = @()
    $lxss = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss'
    if (-not (Test-Path $lxss)) { return @() }
    Get-ChildItem $lxss -ErrorAction SilentlyContinue | ForEach-Object {
        $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        if (-not $p) { return }
        $name = [string]$p.DistributionName
        if (-not $name) { return }
        $base = ConvertTo-UncPath ([string]$p.BasePath)
        $vhdx = Join-Path $base 'ext4.vhdx'
        $bytes = Get-FileSizeBytes $vhdx
        $keep = -not ($name -match 'docker-desktop')
        $distros += [pscustomobject]@{
            name     = $name
            version  = $(if ($p.PSObject.Properties['Version']) { [int]$p.Version } else { 2 })
            basePath = $base
            vhdx     = $(if (Test-Path -LiteralPath $vhdx) { $vhdx } else { $null })
            bytes    = $bytes
            gb       = [math]::Round($bytes / 1GB, 2)
            keep     = $keep
            kind     = $(if ($name -match 'docker-desktop') { 'docker' } else { 'linux' })
        }
    }
    return $distros
}

function Get-DevDriveInfo {
    $vols = @(Get-Volume -ErrorAction SilentlyContinue | Where-Object {
        $_.DriveLetter -and ($_.FileSystem -eq 'ReFS' -or $_.FileSystemLabel -match 'Dev Drive')
    })
    $candidates = @(
        'E:\DevDrive\Dev Drive.vhdx',
        'C:\DevDrive\Dev Drive.vhdx'
    )
    $found = @()
    foreach ($c in $candidates) {
        if (Test-Path -LiteralPath $c) {
            $bytes = Get-FileSizeBytes $c
            $found += [pscustomobject]@{
                path  = $c
                bytes = $bytes
                gb    = [math]::Round($bytes / 1GB, 2)
                onC   = ($c -like 'C:\*')
            }
        }
    }
    $letter = $null
    $label = $null
    $guid = $null
    if ($vols.Count -gt 0) {
        $v = $vols[0]
        $letter = [string]$v.DriveLetter
        $label = [string]$v.FileSystemLabel
        if ($v.UniqueId -match 'Volume\{([^}]+)\}') { $guid = $Matches[1] }
    }
    return [pscustomobject]@{
        present    = ($vols.Count -gt 0 -or $found.Count -gt 0)
        letter     = $letter
        label      = $label
        volumeGuid = $guid
        vhdx       = $found
        keep       = ($vols.Count -gt 0 -or $found.Count -gt 0)
    }
}

function Get-DockerDataVhd {
    $p = Join-Path $env:LOCALAPPDATA 'Docker\wsl\disk\docker_data.vhdx'
    $bytes = Get-FileSizeBytes $p
    [pscustomobject]@{
        present = (Test-Path -LiteralPath $p)
        path    = $p
        bytes   = $bytes
        gb      = [math]::Round($bytes / 1GB, 2)
        keep    = $false
    }
}

function Get-BraveInventory {
    $root = Join-Path $env:LOCALAPPDATA 'BraveSoftware\Brave-Browser\User Data\Default'
    $extDir = Join-Path $root 'Extensions'
    $known = @{
        'aeblfdkhhhdcdjpifhhbdiojplfjncoa' = '1Password'
        'cclelndahbckbenkjhflpdbgdldlbecc' = 'Get cookies.txt LOCALLY'
        'fcoeoabgfenejglbffodgkkbkcdhcgfn' = 'Claude'
        'fjoaledfpmneenckfbpdfhkmimnjocfa' = 'NordVPN'
        'fmkadmapgofadopljbjfkapdkoienihi' = 'React Developer Tools'
        'ghmbeldphafepmbegfdlkpapadhbakde' = 'Proton Pass'
        'gnfldmcodokkpcejgdlffnjakifemick' = 'Imgur Unblocker'
        'jghecgabfgfdldnmbfkhmffcabddioke' = 'Volume Master'
        'kgcjekpmcjjogibpjebkhaanilehneje' = 'Karakeep'
        'ncjedehfkpnliaafimjhdjjeggmfmlgf' = 'Copilot sidebar'
    }
    $exts = @()
    if (Test-Path -LiteralPath $extDir) {
        Get-ChildItem -LiteralPath $extDir -Directory -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -notmatch '^(Temp|internal)' -and $_.Name.Length -eq 32
        } | ForEach-Object {
            $id = $_.Name
            $verDir = Get-ChildItem $_.FullName -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
            $verName = ''
            $name = $known[$id]
            if (-not $name) { $name = $id }
            if ($verDir) {
                $verName = [string]$verDir.Name
                $mf = Join-Path $verDir.FullName 'manifest.json'
                if (Test-Path -LiteralPath $mf) {
                    try {
                        $m = Get-Content -LiteralPath $mf -Raw | ConvertFrom-Json
                        if ($m.name -and $m.name -notmatch '^__MSG_') { $name = [string]$m.name }
                    } catch { }
                }
            }
            $exts += [pscustomobject]@{
                id      = $id
                name    = $name
                version = $verName
                url     = "https://chromewebstore.google.com/detail/$id"
            }
        }
    }
    $bm = Join-Path $root 'Bookmarks'
    [pscustomobject]@{
        present          = (Test-Path -LiteralPath $root)
        profile          = 'Default'
        bookmarksPath    = $(if (Test-Path -LiteralPath $bm) { $bm } else { $null })
        bookmarksBytes   = Get-FileSizeBytes $bm
        extensions       = $exts
        keep             = $true
    }
}

function Get-DotfileInventory {
    $up = $env:USERPROFILE
    $items = @(
        @{ Key = 'wslconfig'; Path = Join-Path $up '.wslconfig' }
        @{ Key = 'gitconfig'; Path = Join-Path $up '.gitconfig' }
        @{ Key = 'sshConfig'; Path = Join-Path $up '.ssh\config' }
        @{ Key = 'sshKnownHosts'; Path = Join-Path $up '.ssh\known_hosts' }
        @{ Key = 'grokConfig'; Path = Join-Path $up '.grok\config.toml' }
        @{ Key = 'terminal'; Path = Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json' }
        @{ Key = 'psProfile'; Path = Join-Path ([Environment]::GetFolderPath('MyDocuments')) 'PowerShell\Microsoft.PowerShell_profile.ps1' }
    )
    $out = @()
    foreach ($i in $items) {
        $out += [pscustomobject]@{
            key     = $i.Key
            path    = $i.Path
            present = [bool](Test-Path -LiteralPath $i.Path)
        }
    }
    [pscustomobject]@{ items = $out; keep = $true }
}

function Get-DestinationVolumes {
    $vols = @()
    Get-Volume -ErrorAction SilentlyContinue | Where-Object {
        $_.DriveLetter -and $_.DriveLetter -ne 'C' -and $_.FileSystem -ne 'ReFS' -and $_.FileSystemLabel -notmatch 'Dev Drive'
    } | Sort-Object { -$_.SizeRemaining } | ForEach-Object {
        $guid = $null
        if ($_.UniqueId -match 'Volume\{([^}]+)\}') { $guid = $Matches[1] }
        $vols += [pscustomobject]@{
            letter    = [string]$_.DriveLetter
            label     = [string]$_.FileSystemLabel
            fileSystem = [string]$_.FileSystem
            gb        = [math]::Round($_.Size / 1GB, 0)
            freeGb    = [math]::Round($_.SizeRemaining / 1GB, 1)
            guid      = $guid
            suggested = ('{0}:\Backups\{1}-{2}' -f $_.DriveLetter, $env:COMPUTERNAME, (Get-Date -Format 'yyyy-MM-dd'))
        }
    }
    return $vols
}

function Get-WindowsWslInventory {
    [pscustomobject]@{
        schemaVersion = 1
        scannedAt     = (Get-Date).ToString('o')
        computer      = $env:COMPUTERNAME
        user          = $env:USERNAME
        userProfile   = $env:USERPROFILE
        linuxProfile  = 'home'  # home | work | skip — not universal
        destinations  = @(Get-DestinationVolumes)
        apps          = @(Get-WingetPackages)
        wsl           = @(Get-WslDistros)
        devDrive      = Get-DevDriveInfo
        docker        = Get-DockerDataVhd
        brave         = Get-BraveInventory
        dotfiles      = Get-DotfileInventory
    }
}

# Dotsource = functions only. Direct run = print JSON.
if ($MyInvocation.InvocationName -ne '.') {
    Get-WindowsWslInventory | ConvertTo-Json -Depth 8
}
