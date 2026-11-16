<#
.SYNOPSIS
  Builds the Windows installer: target\release\bundle\nsis\PSP Sidescreen_<version>_x64-setup.exe

.DESCRIPTION
  1. fetches the virtual display driver (checked against a pinned hash) if it is not there yet,
  2. builds the helper the installer runs (sidescreen-setup.exe),
  3. gathers everything the installer carries into crates\sidescreen-app\resources,
  4. builds the window and the program, and packs them with the resources into the installer.

  Needs Rust, Node.js and the Visual Studio C++ Build Tools. The PSP program is not built here:
  psp\package is committed (see psp\README.md).
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $root

function Step([string] $text) { Write-Host "`n== $text" -ForegroundColor Yellow }
function Check([string] $what) { if ($LASTEXITCODE -ne 0) { throw "$what failed (exit code $LASTEXITCODE)" } }

Step 'The virtual display driver'
if (-not (Test-Path 'drivers\vdd\MttVDD.inf')) { & "$PSScriptRoot\fetch-vdd-driver.ps1" }
else { Write-Host 'already fetched' }

Step 'The helper the installer runs'
cargo build --release -p sidescreen-setup; Check 'cargo build sidescreen-setup'

Step 'Gathering what the installer carries'
$resources = Join-Path $root 'crates\sidescreen-app\resources'
# README.md stays: Tauri needs the folder to exist and not be empty, even to check the code.
if (Test-Path $resources) { Get-ChildItem $resources -Exclude 'README.md' | Remove-Item -Recurse -Force }
New-Item -ItemType Directory -Force "$resources\drivers\vdd", "$resources\drivers\psp", "$resources\psp" | Out-Null
Copy-Item 'target\release\sidescreen-setup.exe' $resources
Copy-Item 'drivers\vdd\*' "$resources\drivers\vdd"
Copy-Item 'drivers\psp\*' "$resources\drivers\psp"
Copy-Item 'psp\package\*' "$resources\psp" -Recurse
Copy-Item 'NOTICE.md' $resources
Get-ChildItem $resources -Recurse -File | Measure-Object -Property Length -Sum |
    ForEach-Object { Write-Host ("{0} files, {1:N1} MB" -f $_.Count, ($_.Sum / 1MB)) }

Step 'The window'
if (-not (Test-Path 'app\node_modules')) { npm ci --prefix app; Check 'npm ci' }

Step 'The program and the installer'
Push-Location 'crates\sidescreen-app'
try {
    & '..\..\app\node_modules\.bin\tauri.cmd' build --bundles nsis
    Check 'tauri build'
}
finally { Pop-Location }

$installer = Get-ChildItem 'target\release\bundle\nsis\*-setup.exe' | Sort-Object LastWriteTime | Select-Object -Last 1
Step 'Done'
Write-Host $installer.FullName
Write-Host ("{0:N1} MB, SHA-256 {1}" -f ($installer.Length / 1MB), (Get-FileHash $installer.FullName).Hash)
