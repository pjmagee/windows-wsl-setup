#Requires -Version 5.1
<#
.SYNOPSIS
  Windows-host bootstrap for an existing work laptop: Ubuntu 26.04 + Terminal.

.DESCRIPTION
  Does not install the Linux toolchain (that is install.sh). This script:
  - enables WSL 2 and Ubuntu-26.04 if they are missing
  - NOPASSWD sudo for the user already on that distro (wsl -u root)
  - makes `wsl` and `ubuntu` open that distro at ~
  - points Windows Terminal at those profiles (Ubuntu is the default)

  Does not use cloud-init. Does not invent or lock a Linux user.
  If Ubuntu is brand new, finish the normal username/password prompt once
  (Microsoft: then it auto-signs-in), then re-run this script.

  Re-run safe. Prefer a normal (non-admin) PowerShell unless wsl --install
  asks for elevation.

.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File .\windows\bootstrap.ps1
#>
[CmdletBinding()]
param(
    [string] $UserName,
    [string] $Distro = 'Ubuntu-26.04',
    [switch] $SkipLinuxInstall
)

$ErrorActionPreference = 'Stop'
$UbuntuProfileGuid = '{8f3e1c2a-9b74-4d6e-a1f0-2c8e4e6b90aa}'
$WslProfileGuid    = '{8f3e1c2a-9b74-4d6e-a1f0-2c8e4e6b90bb}'

function Write-Step { param([string]$Message) Write-Host "`n==> $Message" }

function Get-LinuxUserName {
    param([string] $Raw)
    $n = $Raw.ToLowerInvariant() -replace '\s+', '' -replace '[^a-z0-9_-]', ''
    if ($n -notmatch '^[a-z_]') { $n = 'u' + $n }
    if ($n.Length -gt 32) { $n = $n.Substring(0, 32) }
    if ([string]::IsNullOrWhiteSpace($n)) { $n = 'ubuntu' }
    return $n
}

function Get-WslDistroNames {
    $prev = [Console]::OutputEncoding
    try {
        [Console]::OutputEncoding = [System.Text.Encoding]::Unicode
        $lines = @(wsl.exe -l -q 2>$null)
    } finally {
        [Console]::OutputEncoding = $prev
    }
    $lines | ForEach-Object { ($_ -replace [char]0, '').Trim() } | Where-Object { $_ }
}

function Test-WslDistro {
    param([string] $Name)
    (Get-WslDistroNames) -contains $Name
}

function Get-ExistingLinuxUser {
    param([string] $Name)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $out = wsl.exe -d $Name -u root -- bash -lc "getent passwd 1000 | cut -d: -f1" 2>$null
    $ErrorActionPreference = $prev
    if ($out) { return (("$out" -replace [char]0, '').Trim()) }
    return ''
}

function Set-IniValue {
    param(
        [string] $Path,
        [string] $Section,
        [string] $Key,
        [string] $Value
    )
    $dir = Split-Path -Parent $Path
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    [string[]] $lines = @()
    if (Test-Path $Path) {
        $lines = @(Get-Content -LiteralPath $Path)
    }
    $sectionHeader = "[$Section]"
    $idx = [Array]::FindIndex($lines, [Predicate[string]]{ param($l) $l.Trim() -eq $sectionHeader })
    if ($idx -lt 0) {
        if ($lines.Count -gt 0 -and $lines[-1].Trim() -ne '') { $lines += '' }
        $lines += $sectionHeader
        $lines += "$Key=$Value"
        Set-Content -LiteralPath $Path -Value $lines -Encoding ASCII
        return
    }
    $end = $idx + 1
    while ($end -lt $lines.Count -and $lines[$end] -notmatch '^\s*\[') { $end++ }
    $replaced = $false
    for ($i = $idx + 1; $i -lt $end; $i++) {
        if ($lines[$i] -match ("^\s*" + [regex]::Escape($Key) + "\s*=")) {
            $lines[$i] = "$Key=$Value"
            $replaced = $true
            break
        }
    }
    if (-not $replaced) {
        $before = @()
        if ($idx -ge 0) { $before = $lines[0..$idx] }
        $mid = @()
        if ($end - $idx -gt 1) { $mid = $lines[($idx + 1)..($end - 1)] }
        $after = @()
        if ($end -lt $lines.Count) { $after = $lines[$end..($lines.Count - 1)] }
        $lines = $before + $mid + @("$Key=$Value") + $after
    }
    Set-Content -LiteralPath $Path -Value $lines -Encoding ASCII
}

function Ensure-WslConfig {
    Write-Step "WSLg (.wslconfig)"
    $path = Join-Path $env:USERPROFILE '.wslconfig'
    Set-IniValue -Path $path -Section 'wsl2' -Key 'guiApplications' -Value 'true'
}

function Ensure-WslFeature {
    Write-Step "WSL 2"
    $wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
    if (-not $wsl) {
        Write-Host "wsl.exe is missing. Enabling WSL (may require elevation + reboot)."
        wsl.exe --install --no-distribution
        throw "WSL was just installed. Reboot Windows, then re-run windows/bootstrap.ps1."
    }
    try { wsl.exe --set-default-version 2 | Out-Null } catch { }
    $ver = wsl.exe --version 2>&1 | Out-String
    if ($ver -match 'must be updated|not supported|Windows Subsystem for Linux') {
        Write-Host $ver
    }
}

function Ensure-Distro {
    if (Test-WslDistro $Distro) {
        Write-Step "$Distro already installed"
        return
    }
    $online = (wsl.exe --list --online 2>&1 | Out-String) -replace [char]0, ''
    if ($online -notmatch [regex]::Escape($Distro)) {
        throw "$Distro is not in ``wsl --list --online``. Refusing to install a different Ubuntu."
    }
    Write-Step "install $Distro (normal WSL install; complete the username/password prompt if it appears)"
    $attempts = @(
        @('--install', $Distro),
        @('--install', '-d', $Distro)
    )
    foreach ($installArgs in $attempts) {
        $prev = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        & wsl.exe @installArgs
        $ErrorActionPreference = $prev
        if (Test-WslDistro $Distro) { break }
    }
    if (-not (Test-WslDistro $Distro)) {
        throw "wsl --install $Distro did not register the distro. Re-run elevated if Windows asked for admin."
    }
}

function Ensure-PasswordlessSudo {
    param([string] $LinuxUser)
    Write-Step "NOPASSWD sudo for existing user ($LinuxUser)"
    $script = Join-Path $PSScriptRoot 'ensure-user.sh'
    if (-not (Test-Path $script)) { throw "missing $script" }
    Get-Content -LiteralPath $script -Raw | wsl.exe -d $Distro -u root -- bash -s -- $LinuxUser
    if ($LASTEXITCODE -ne 0) {
        throw "No Linux user on $Distro yet. Open Ubuntu once, create the username/password WSL asks for, then re-run bootstrap.ps1."
    }
    wsl.exe --terminate $Distro 2>$null | Out-Null
    $who = (wsl.exe -d $Distro -- whoami 2>$null)
    $who = (("$who" -replace [char]0, '').Trim())
    if (-not $who) { throw "distro launched but whoami was empty" }
    Write-Host "default user: $who"
    $sudo = (wsl.exe -d $Distro -- bash -lc 'sudo -n true && echo sudo-ok' 2>$null)
    $sudo = (("$sudo" -replace [char]0, '').Trim())
    if ($sudo -ne 'sudo-ok') {
        throw "sudo still requires a password for $who"
    }
}

function Ensure-DefaultDistro {
    Write-Step "default WSL distro = $Distro"
    wsl.exe --set-default $Distro
}

function Get-UserPathEntries {
    $raw = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ([string]::IsNullOrWhiteSpace($raw)) { return @() }
    $raw.Split(';') | Where-Object { $_ }
}

function Ensure-UbuntuShim {
    Write-Step "ubuntu launcher on user PATH"
    $bin = Join-Path $env:USERPROFILE '.wsl-setup\bin'
    if (-not (Test-Path $bin)) { New-Item -ItemType Directory -Path $bin | Out-Null }
    $src = Join-Path $PSScriptRoot 'ubuntu.cmd'
    Copy-Item -LiteralPath $src -Destination (Join-Path $bin 'ubuntu.cmd') -Force

    $entries = @(Get-UserPathEntries)
    if ($entries -notcontains $bin) {
        $new = @($bin) + $entries
        [Environment]::SetEnvironmentVariable('Path', ($new -join ';'), 'User')
    }
    if ($env:Path -notlike "*$bin*") {
        $env:Path = "$bin;$env:Path"
    }
}

function Get-TerminalSettingsPaths {
    $paths = @(
        (Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json'),
        (Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json'),
        (Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\settings.json')
    )
    $paths | Where-Object { Test-Path $_ }
}

function Ensure-WindowsTerminal {
    Write-Step "Windows Terminal profiles (Ubuntu, wsl) + default"
    $fragDir = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\Fragments\wsl-setup'
    if (-not (Test-Path $fragDir)) { New-Item -ItemType Directory -Path $fragDir | Out-Null }
    $fragment = @{
        profiles = @(
            @{
                guid              = $UbuntuProfileGuid
                name              = 'Ubuntu'
                commandline       = "wsl.exe -d $Distro ~"
                startingDirectory = '~'
                hidden            = $false
                icon              = 'ms-appx:///ProfileIcons/{9acb9455-ca41-5af7-950f-6bca1bc9722f}.png'
            }
            @{
                guid              = $WslProfileGuid
                name              = 'wsl'
                commandline       = "wsl.exe -d $Distro ~"
                startingDirectory = '~'
                hidden            = $false
                icon              = 'ms-appx:///ProfileIcons/{9acb9455-ca41-5af7-950f-6bca1bc9722f}.png'
            }
        )
    }
    $json = $fragment | ConvertTo-Json -Depth 6
    Set-Content -LiteralPath (Join-Path $fragDir 'profiles.json') -Value $json -Encoding UTF8

    $settings = @(Get-TerminalSettingsPaths)
    if ($settings.Count -eq 0) {
        $store = Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState'
        $unpackaged = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal'
        $targetDir = if (Test-Path $store) { $store } else { $unpackaged }
        if (-not (Test-Path $targetDir)) { New-Item -ItemType Directory -Path $targetDir | Out-Null }
        $minimal = @"
{
    "`$help": "https://aka.ms/terminal-documentation",
    "`$schema": "https://aka.ms/terminal-profiles-schema",
    "defaultProfile": "$UbuntuProfileGuid",
    "profiles": { "defaults": {}, "list": [] }
}
"@
        Set-Content -LiteralPath (Join-Path $targetDir 'settings.json') -Value $minimal -Encoding UTF8
        return
    }
    foreach ($path in $settings) {
        $raw = Get-Content -LiteralPath $path -Raw
        if ($raw -match '"defaultProfile"\s*:') {
            $raw = [regex]::Replace($raw, '"defaultProfile"\s*:\s*"[^"]*"', "`"defaultProfile`": `"$UbuntuProfileGuid`"")
        } else {
            $raw = $raw -replace '\{', "{`r`n    `"defaultProfile`": `"$UbuntuProfileGuid`","
        }
        Set-Content -LiteralPath $path -Value $raw -Encoding UTF8
    }
}

function Update-PsProfile {
    param([string] $Path)
    $dir = Split-Path -Parent $Path
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $block = @'
# >>> wsl-setup >>>
function wsl {
    if ($args.Count -eq 0) { & wsl.exe ~ } else { & wsl.exe @args }
}
function ubuntu { & wsl.exe -d Ubuntu-26.04 ~ @args }
# <<< wsl-setup <<<
'@
    $existing = ''
    if (Test-Path $Path) { $existing = Get-Content -LiteralPath $Path -Raw }
    if ($existing -match '>>> wsl-setup >>>') {
        $existing = [regex]::Replace($existing, '(?s)# >>> wsl-setup >>>.*?# <<< wsl-setup <<<\r?\n?', '')
    }
    $existing = $existing.TrimEnd()
    if ($existing) { $existing = $existing + "`r`n`r`n" }
    Set-Content -LiteralPath $Path -Value ($existing + $block + "`r`n") -Encoding UTF8
}

function Ensure-PowerShellLaunchers {
    Write-Step "PowerShell functions: wsl / ubuntu -> Ubuntu-26.04 at ~"
    $docs = [Environment]::GetFolderPath('MyDocuments')
    Update-PsProfile (Join-Path $docs 'PowerShell\Microsoft.PowerShell_profile.ps1')
    Update-PsProfile (Join-Path $docs 'WindowsPowerShell\Microsoft.PowerShell_profile.ps1')
}

function Invoke-LinuxInstall {
    if ($SkipLinuxInstall) {
        Write-Host "skipping install.sh (-SkipLinuxInstall)"
        return
    }
    Write-Step "clone/update repo on the Linux disk and run install.sh"
    $linuxRepo = '~/code/wsl-setup'
    $winRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
    $remote = 'https://github.com/pjmagee/wsl-setup.git'
    $wslWin = (wsl.exe -d $Distro -- wslpath -a $winRoot 2>$null)
    $wslWin = (("$wslWin" -replace [char]0, '').Trim())
    $setup = @"
set -euo pipefail
sudo -n apt-get update -y
sudo -n DEBIAN_FRONTEND=noninteractive apt-get install -y git curl
mkdir -p `$HOME/code
if [ ! -d $linuxRepo/.git ]; then
  if [ -n '$wslWin' ] && [ -d '$wslWin/.git' ]; then
    git clone '$wslWin' $linuxRepo
  else
    git clone $remote $linuxRepo
  fi
else
  git -C $linuxRepo pull --ff-only || true
fi
chmod +x $linuxRepo/install.sh $linuxRepo/scripts/wsl-open $linuxRepo/windows/ensure-user.sh
cd $linuxRepo
./install.sh
"@
    wsl.exe -d $Distro --cd '~' -- bash -lc $setup
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "install.sh failed. Repo on Windows is at $winRoot — fix inside WSL and re-run ./install.sh"
    }
}

# --- main ---
Ensure-WslConfig
Ensure-WslFeature
Ensure-Distro
$linuxUser = Get-ExistingLinuxUser $Distro
if (-not $linuxUser) {
    if ($UserName) { $linuxUser = Get-LinuxUserName $UserName }
}
if (-not $linuxUser) {
    throw "No Linux user on $Distro. Open Ubuntu once, create the username/password WSL asks for, then re-run bootstrap.ps1."
}
Ensure-PasswordlessSudo -LinuxUser $linuxUser
Ensure-DefaultDistro
Ensure-UbuntuShim
Ensure-PowerShellLaunchers
Ensure-WindowsTerminal
Invoke-LinuxInstall

Write-Host @"

Done.

From Windows Terminal:
  - new tab defaults to Ubuntu at ~
  - profiles ``Ubuntu`` and ``wsl`` are the same session
  - ``ubuntu`` (new terminals) and a bare ``wsl`` (cmd, after PATH refresh) also land at ~

Open a new Windows Terminal window so the profile + PATH changes load.
From a Linux path:  code .

Host leftovers this script does not install: Docker Desktop, VS Code + WSL
extension, 1Password for Windows (SSH agent).
"@
