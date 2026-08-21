#Requires -Version 5.1
# wwm.exe -> %USERPROFILE%\.wwm (Windows 11 x86_64, PowerShell 5.1)
# irm https://pjmagee.github.io/wwm/install.txt | iex
#
# Optional before iex: $WwmExeUrl = 'https://.../wwm.exe'

& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    if ($env:WSL_DISTRO_NAME) {
        throw 'Run this from Windows PowerShell, not WSL.'
    }

    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    if ($arch -ne 'AMD64') {
        throw 'wwm is x86_64 only.'
    }

    [Net.ServicePointManager]::SecurityProtocol =
        [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $url = 'https://github.com/pjmagee/wwm/releases/latest/download/wwm.exe'
    if ($WwmExeUrl) { $url = [string]$WwmExeUrl }

    $root = Join-Path $env:USERPROFILE '.wwm'
    $exe = Join-Path $root 'wwm.exe'
    if (-not (Test-Path -LiteralPath $root)) {
        New-Item -ItemType Directory -Path $root | Out-Null
    }

    Write-Host "Downloading $url"
    Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $exe

    $fs = [IO.File]::OpenRead($exe)
    try {
        $mz = New-Object byte[] 2
        $n = $fs.Read($mz, 0, 2)
        if ($n -ne 2 -or $mz[0] -ne 0x4D -or $mz[1] -ne 0x5A) {
            throw "Download was not wwm.exe (404?). $url"
        }
    } finally {
        $fs.Close()
    }

    $oldBin = Join-Path $env:USERPROFILE '.wsl-setup\bin'
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @()
    if ($userPath) {
        $entries = @($userPath.Split(';') | Where-Object { $_ -and $_ -ne $oldBin })
    }
    if ($entries -notcontains $root) {
        $entries = @($root) + $entries
        [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')
    }
    if ($env:Path -notlike "*$root*") {
        $env:Path = "$root;$env:Path"
    }

    Write-Host "Installed $exe"
}
