#Requires -Version 5.1
<#
.SYNOPSIS
  Windows-host bootstrap for an existing work laptop: Ubuntu 26.04 + Terminal.

.DESCRIPTION
  Does not install the Linux toolchain (that is install.sh). This script:
  - updates WSL if it can (Ubuntu-26.04 needs WSL 2.4.10+)
  - enables WSL 2 and Ubuntu-26.04 if they are missing
  - leaves any other distro (Ubuntu-24.04, Store Ubuntu, docker-desktop) installed
  - NOPASSWD sudo for the user already on that distro (wsl -u root)
  - makes `wsl` and `ubuntu` open that distro at ~
  - points Windows Terminal at a tab per WSL distro (Ubuntu is the default)

  Does not use cloud-init. Does not invent or lock a Linux user.
  Does not wsl --unregister anything.
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

function Write-InstalledDistros {
    Write-Step "installed WSL distros (will not unregister any)"
    $prev = [Console]::OutputEncoding
    try {
        [Console]::OutputEncoding = [System.Text.Encoding]::Unicode
        wsl.exe -l -v
    } finally {
        [Console]::OutputEncoding = $prev
    }
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

    Write-Host "wsl --update (Ubuntu-26.04's .wsl image wants WSL 2.4.10+)."
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    wsl.exe --update
    $ErrorActionPreference = $prev

    $ver = wsl.exe --version 2>&1 | Out-String
    $ver = $ver -replace [char]0, ''
    Write-Host $ver.Trim()
    if ($ver -match 'WSL version:\s*(\d+)\.(\d+)\.(\d+)') {
        $maj = [int]$Matches[1]
        $min = [int]$Matches[2]
        $pat = [int]$Matches[3]
        $tooOld = $false
        if ($maj -lt 2) { $tooOld = $true }
        elseif ($maj -eq 2 -and $min -lt 4) { $tooOld = $true }
        elseif ($maj -eq 2 -and $min -eq 4 -and $pat -lt 10) { $tooOld = $true }
        if ($tooOld) {
            Write-Warning "WSL $maj.$min.$pat is older than 2.4.10. Ubuntu-26.04 may be missing from wsl --list --online. Re-run elevated: wsl --update"
        }
    } elseif ($ver -match 'must be updated|not supported') {
        Write-Warning $ver.Trim()
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
    $raw = Get-Content -LiteralPath $script -Raw
    if ([string]::IsNullOrWhiteSpace($raw)) { throw "empty $script" }
    # Windows checkouts may still be CRLF if .gitattributes was not applied.
    $unix = $raw -replace "`r`n", "`n" -replace "`r", "`n"
    $unix | wsl.exe -d $Distro -u root -- bash -s -- $LinuxUser
    if ($LASTEXITCODE -ne 0) {
        throw "ensure-user.sh failed on $Distro (no Linux user yet, or the script did not run). Open Ubuntu once, create the username/password WSL asks for, then re-run bootstrap.ps1."
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
    $root = Join-Path $env:USERPROFILE '.wwm'
    if (-not (Test-Path $root)) { New-Item -ItemType Directory -Path $root | Out-Null }
    $src = Join-Path $PSScriptRoot 'ubuntu.cmd'
    Copy-Item -LiteralPath $src -Destination (Join-Path $root 'ubuntu.cmd') -Force

    $oldBin = Join-Path $env:USERPROFILE '.wsl-setup\bin'
    $entries = @(Get-UserPathEntries | Where-Object { $_ -and $_ -ne $oldBin })
    if ($entries -notcontains $root) {
        $entries = @($root) + $entries
    }
    [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')
    if ($env:Path -notlike "*$root*") {
        $env:Path = "$root;$env:Path"
    }
}

function Get-OfficialWslProfileGuid {
    param([string] $Name)
    foreach ($path in @(Get-TerminalSettingsPaths)) {
        $raw = Get-Content -LiteralPath $path -Raw
        try {
            $j = $raw | ConvertFrom-Json
        } catch {
            continue
        }
        $list = @()
        if ($j.profiles -and $j.profiles.list) { $list = @($j.profiles.list) }
        foreach ($p in $list) {
            if ($p.name -eq $Name -and $p.source -eq 'Microsoft.WSL' -and $p.guid) {
                return [string]$p.guid
            }
        }
    }
    return ''
}

function Get-TerminalSettingsPaths {
    $paths = @(
        (Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json'),
        (Join-Path $env:LOCALAPPDATA 'Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json'),
        (Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\settings.json')
    )
    $paths | Where-Object { Test-Path $_ }
}

function Set-TerminalDefaultProfile {
    param([string] $Guid)
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
    "defaultProfile": "$Guid",
    "disabledProfileSources": [ "Windows.Terminal.Wsl" ],
    "profiles": { "defaults": {}, "list": [] }
}
"@
        Set-Content -LiteralPath (Join-Path $targetDir 'settings.json') -Value $minimal -Encoding UTF8
        return
    }
    foreach ($path in $settings) {
        $raw = Get-Content -LiteralPath $path -Raw
        if ($raw -notmatch 'disabledProfileSources') {
            $raw = $raw -replace '\{', "{`r`n    `"disabledProfileSources`": [ `"Windows.Terminal.Wsl`" ],"
        }
        if ($raw -match '"defaultProfile"\s*:') {
            $raw = [regex]::Replace($raw, '"defaultProfile"\s*:\s*"[^"]*"', "`"defaultProfile`": `"$Guid`"")
        } else {
            $raw = $raw -replace '\{', "{`r`n    `"defaultProfile`": `"$Guid`","
        }
        Set-Content -LiteralPath $path -Value $raw -Encoding UTF8
    }
}

function Ensure-WindowsTerminal {
    Write-Step "Windows Terminal profiles (one tab per WSL distro)"
    $wwm = @(
        (Join-Path $env:USERPROFILE '.wwm\wwm.exe'),
        (Join-Path $PSScriptRoot 'cli\target\release\wwm.exe'),
        (Join-Path $PSScriptRoot 'cli\target\debug\wwm.exe')
    ) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if ($wwm) {
        & $wwm distro sync
        if ($LASTEXITCODE -ne 0) { throw "wwm distro sync failed" }
        $official = Get-OfficialWslProfileGuid $Distro
        if ($official) { Set-TerminalDefaultProfile $official }
        return
    }
    $fragDir = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\Fragments\wwm'
    if (-not (Test-Path $fragDir)) { New-Item -ItemType Directory -Path $fragDir | Out-Null }
    $oldFrag = Join-Path $env:LOCALAPPDATA 'Microsoft\Windows Terminal\Fragments\wsl-setup'
    if (Test-Path $oldFrag) { Remove-Item -LiteralPath $oldFrag -Recurse -Force }
    $penguin = 'ms-appx:///ProfileIcons/{9acb9455-ca41-5af7-950f-6bca1bc9722f}.png'
    $fragment = @{
        profiles = @(
            @{
                guid              = $WslProfileGuid
                name              = 'wsl'
                commandline       = "$env:SystemRoot\System32\wsl.exe"
                startingDirectory = '~'
                hidden            = $false
                icon              = $penguin
            }
        )
    }
    $json = $fragment | ConvertTo-Json -Depth 6
    Set-Content -LiteralPath (Join-Path $fragDir 'profiles.json') -Value $json -Encoding UTF8
    $official = Get-OfficialWslProfileGuid $Distro
    if ($official) { Set-TerminalDefaultProfile $official }
}

function Update-PsProfile {
    param([string] $Path)
    $dir = Split-Path -Parent $Path
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
    $block = @'
# >>> wwm >>>
function wsl {
    if ($args.Count -eq 0) { & wsl.exe ~ } else { & wsl.exe @args }
}
function ubuntu { & wsl.exe -d Ubuntu-26.04 ~ @args }
# <<< wwm <<<
'@
    $existing = ''
    if (Test-Path $Path) { $existing = Get-Content -LiteralPath $Path -Raw }
    if ($existing -match '>>> wsl-setup >>>') {
        $existing = [regex]::Replace($existing, '(?s)# >>> wsl-setup >>>.*?# <<< wsl-setup <<<\r?\n?', '')
    }
    if ($existing -match '>>> wwm >>>') {
        $existing = [regex]::Replace($existing, '(?s)# >>> wwm >>>.*?# <<< wwm <<<\r?\n?', '')
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
    $linuxRepo = '~/code/wwm'
    $legacyRepo = '~/code/windows-wsl-manager'
    $winRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
    $remote = 'https://github.com/pjmagee/wwm.git'
    $wslWin = (wsl.exe -d $Distro -- wslpath -a $winRoot 2>$null)
    $wslWin = (("$wslWin" -replace [char]0, '').Trim())
    $setup = @"
set -euo pipefail
sudo -n apt-get update -y
sudo -n DEBIAN_FRONTEND=noninteractive apt-get install -y git curl
mkdir -p `$HOME/code
if [ ! -d $linuxRepo/.git ]; then
  if [ -d $legacyRepo/.git ]; then
    git clone $legacyRepo $linuxRepo
  elif [ -n '$wslWin' ] && [ -d '$wslWin/.git' ]; then
    git clone '$wslWin' $linuxRepo
  else
    git clone $remote $linuxRepo
  fi
else
  git -C $linuxRepo pull --ff-only || true
fi
git -C $linuxRepo remote set-url origin $remote 2>/dev/null || true
chmod +x $linuxRepo/install.sh $linuxRepo/scripts/wsl-open $linuxRepo/windows/ensure-user.sh
cd $linuxRepo
./install.sh work
"@
    wsl.exe -d $Distro --cd '~' -- bash -lc $setup
    if ($LASTEXITCODE -ne 0) {
        throw "install.sh failed inside $Distro. Windows clone is $winRoot. Fix inside WSL and re-run ./install.sh (or re-run this script)."
    }
}

# --- main ---
Ensure-WslConfig
Ensure-WslFeature
Write-InstalledDistros
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
  - ``ubuntu`` in a new PowerShell session and a bare ``wsl`` (after PATH refresh) land at ~
  - ``ubuntu`` in cmd may still be Store ubuntu.exe if that alias exists

Open a new Windows Terminal window so the profile + PATH changes load.
From a Linux path:  code .

Other WSL distros (Ubuntu-24.04, Store Ubuntu, docker-desktop) were left
installed. This script never unregisters them.

Host leftovers this script does not install: Docker Desktop (enable WSL
integration for Ubuntu-26.04 — it may still be pointed at 24.04), VS Code +
WSL extension, 1Password for Windows (SSH agent).
"@
