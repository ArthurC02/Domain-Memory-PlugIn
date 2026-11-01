# Installs the Domain Memory executable of one release and checks it against
# that release's SHA256SUMS before it lands.
#
#   & ([scriptblock]::Create((irm https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/download/v6.5.0/install.ps1))) -Version v6.5.0
#
# -Version <tag>  the release to install (default: the latest release)
# -To <dir>       where the executable lands (default: %LOCALAPPDATA%\Programs\domain-memory\bin)
# DOMAIN_MEMORY_DOWNLOAD_URL replaces the release's download address, for a mirror.
param(
    [string]$Version = 'latest',
    [string]$To = (Join-Path $env:LOCALAPPDATA 'Programs\domain-memory\bin')
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    throw 'install.ps1: releases carry an executable for Windows on x86-64 only; build one from a clone with cargo build --release --locked --manifest-path engine/Cargo.toml'
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
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$asset" -OutFile (Join-Path $work $asset)
    Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile (Join-Path $work 'SHA256SUMS')
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
Write-Output "Installed $Version of Domain Memory at $target"
Write-Output "Git hooks find it with: git config domain-memory.executable '$($target -replace '\\', '/')'"
