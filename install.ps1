# Installs the Domain Memory executable of one release, after checking it
# against that release's SHA256SUMS, and puts it on the user's PATH.
#
#   irm https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/latest/download/install.ps1 | iex
#
# Options, as a parameter or in the environment:
#   -Version <tag>  DOMAIN_MEMORY_VERSION         the release (default: the latest)
#   -To <dir>       DOMAIN_MEMORY_INSTALL_DIR     where it lands (default: %LOCALAPPDATA%\Programs\domain-memory\bin)
#                   DOMAIN_MEMORY_NO_MODIFY_PATH  set to leave PATH alone
#                   DOMAIN_MEMORY_SHA256SUMS_SHA256  the digest SHA256SUMS must have, to pin it too
# so a pinned install reads: $env:DOMAIN_MEMORY_VERSION = 'v7.0.3'; irm <url> | iex
# DOMAIN_MEMORY_DOWNLOAD_URL replaces the release's download address, for a mirror.
param(
    [string]$Version = $(if ($env:DOMAIN_MEMORY_VERSION) { $env:DOMAIN_MEMORY_VERSION } else { 'latest' }),
    [string]$To = $(if ($env:DOMAIN_MEMORY_INSTALL_DIR) { $env:DOMAIN_MEMORY_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\domain-memory\bin' })
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    throw 'install.ps1: releases carry an executable for Windows on x86-64 only; build one from a clone with cargo build --release --locked --manifest-path engine/Cargo.toml'
}
if ($Version -match '^[0-9]') { $Version = "v$Version" }
if ($env:DOMAIN_MEMORY_SHA256SUMS_SHA256 -and $Version -eq 'latest') {
    throw "install.ps1: DOMAIN_MEMORY_SHA256SUMS_SHA256 pins one release's checksums, so the next release would break it; set DOMAIN_MEMORY_VERSION to that release too"
}
$asset = 'domain-memory-windows-x86_64.exe'
$releases = 'https://github.com/ArthurC02/Domain-Memory-PlugIn/releases'
$base = $env:DOMAIN_MEMORY_DOWNLOAD_URL
if (-not $base) {
    if ($Version -eq 'latest') { $base = "$releases/latest/download" } else { $base = "$releases/download/$Version" }
}

$work = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    foreach ($file in $asset, 'SHA256SUMS') {
        try {
            Invoke-WebRequest -UseBasicParsing -Uri "$base/$file" -OutFile (Join-Path $work $file)
        } catch {
            throw "install.ps1: could not download $file from $base ($($_.Exception.Message)); is $Version a release? The releases are listed at $releases"
        }
    }
    $sums = (Get-FileHash -Algorithm SHA256 (Join-Path $work 'SHA256SUMS')).Hash
    if ($env:DOMAIN_MEMORY_SHA256SUMS_SHA256 -and $sums -ne $env:DOMAIN_MEMORY_SHA256SUMS_SHA256) {
        throw "install.ps1: SHA256SUMS from $base is not the one DOMAIN_MEMORY_SHA256SUMS_SHA256 pins (pinned $($env:DOMAIN_MEMORY_SHA256SUMS_SHA256), got $($sums.ToLower())); nothing was installed"
    }
    $wanted = Get-Content (Join-Path $work 'SHA256SUMS') |
        ForEach-Object { $fields = -split $_; if ($fields.Count -eq 2 -and $fields[1].TrimStart('*') -eq $asset) { $fields[0] } } |
        Select-Object -First 1
    $actual = (Get-FileHash -Algorithm SHA256 (Join-Path $work $asset)).Hash
    if (-not $wanted -or $wanted -ne $actual) {
        throw "install.ps1: $asset from $base does not match its SHA256SUMS entry ($wanted); nothing was installed"
    }
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $target = Join-Path $To 'domain-memory.exe'
    Move-Item -Force (Join-Path $work $asset) $target
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}

$installed = & $target --version 2>$null
if (-not $installed) { $installed = "Domain Memory $Version" }
Write-Output "Installed $installed at $target"
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -contains $To) {
    Write-Output 'Try it: domain-memory readiness'
} elseif ($env:DOMAIN_MEMORY_NO_MODIFY_PATH) {
    Write-Output "$To is not on PATH; add it to run domain-memory by name and let git hooks find it"
} else {
    [Environment]::SetEnvironmentVariable('Path', (@($userPath, $To) | Where-Object { $_ }) -join ';', 'User')
    $env:Path = "$env:Path;$To"
    Write-Output "Added $To to your PATH; open a new terminal, then try: domain-memory readiness"
}
