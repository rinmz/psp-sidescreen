<#
.SYNOPSIS
  Fetches the signed Virtual Display Driver package that the installer carries.

.DESCRIPTION
  Downloads a pinned release of the open-source Virtual Display Driver
  (https://github.com/VirtualDrivers/Virtual-Display-Driver, MIT licence), refuses to continue
  unless its SHA-256 is the pinned value, and extracts only the 64-bit driver files into
  drivers\vdd. The files are signed by SignPath and are not stored in this repository; this
  script is the record of exactly which package is used.

  The release folder is named "x86", but its INF targets NTamd64, so it is the 64-bit driver.
#>
[CmdletBinding()]
param(
    [string] $Destination
)

$ErrorActionPreference = 'Stop'

# The default cannot be a parameter default: $PSScriptRoot is empty there in Windows PowerShell 5.1.
if (-not $Destination) { $Destination = Join-Path $PSScriptRoot '..\drivers\vdd' }

$version = '25.7.23'
$sha256 = 'A701F2272E9FCF382849B24F913C6DD07597B3B1116525F2E90182F019609154'
$url = "https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/download/$version/VDD.Control.$version.zip"
$folderInZip = 'SignedDrivers/x86/VDD/'

$work = Join-Path ([IO.Path]::GetTempPath()) ("sidescreen-vdd-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $zip = Join-Path $work 'package.zip'
    Write-Host "Downloading Virtual Display Driver $version..."
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing

    $actual = (Get-FileHash -Path $zip -Algorithm SHA256).Hash
    if ($actual -ne $sha256) {
        throw "The downloaded package does not match the pinned hash.`n  expected $sha256`n  actual   $actual"
    }
    Write-Host 'Hash verified.'

    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead($zip)
    try {
        $entries = $archive.Entries | Where-Object { $_.FullName.StartsWith($folderInZip) -and $_.Name }
        if (-not $entries) { throw "No driver files found under $folderInZip in the package." }
        foreach ($entry in $entries) {
            $target = Join-Path $Destination $entry.Name
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $true)
        }
    }
    finally {
        $archive.Dispose()
    }

    $resolved = (Resolve-Path $Destination).Path
    Write-Host "Driver files are in $resolved"
    Get-ChildItem $resolved | Select-Object Name, Length | Format-Table -AutoSize
}
finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
