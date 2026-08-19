#Requires -Version 5.1
<#
  Maintainer helper: build + launch the exe. End users download windows-wsl-setup.exe.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$cli = Join-Path $PSScriptRoot '..\cli'
$manifest = Join-Path $cli 'Cargo.toml'
if (-not (Test-Path -LiteralPath $manifest)) { throw "missing $manifest" }

$cargoHome = Join-Path $env:USERPROFILE '.cargo\bin'
if (Test-Path $cargoHome) { $env:Path = "$cargoHome;$env:Path" }

$target = Join-Path $env:LOCALAPPDATA 'wsl-setup-cli-target'
$env:CARGO_TARGET_DIR = $target
$exe = Join-Path $target 'release\windows-wsl-setup.exe'

$needBuild = $true
if (Test-Path -LiteralPath $exe) {
    $exeTime = (Get-Item -LiteralPath $exe).LastWriteTimeUtc
    $srcTime = Get-ChildItem -LiteralPath (Join-Path $cli 'src') -Recurse -File |
        Measure-Object -Property LastWriteTimeUtc -Maximum |
        Select-Object -ExpandProperty Maximum
    if ($srcTime -and $exeTime -ge $srcTime) { $needBuild = $false }
}

if ($needBuild) {
    Write-Host 'Building Windows WSL Setup TUI (release)…'
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) { throw 'cargo not on PATH. Install Rust (rustup) and re-run.' }
    & cargo build --release --manifest-path $manifest
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}

$env:WSL_SETUP_ROOT = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
& $exe capture
exit $LASTEXITCODE
