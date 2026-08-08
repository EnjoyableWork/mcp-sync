param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$Target,

    [Parameter(Mandatory = $true)]
    [string]$Executable,

    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw 'Release version must be a stable semantic version.'
}
if ($Target -notin @('aarch64-pc-windows-msvc', 'x86_64-pc-windows-msvc')) {
    throw 'Unsupported Windows release target.'
}

$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$reportedVersion = (& $resolvedExecutable --version).Trim()
if ($LASTEXITCODE -ne 0 -or $reportedVersion -ne "mcp-sync $Version") {
    throw 'Release executable version does not match the requested archive version.'
}

New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
$resolvedOutput = (Resolve-Path -LiteralPath $OutputDirectory).Path
$stage = Join-Path ([System.IO.Path]::GetTempPath()) ("mcp-sync-release-package-" + [guid]::NewGuid().ToString('N'))
$archiveName = "mcp-sync-v$Version-$Target.zip"
$archivePath = Join-Path $resolvedOutput $archiveName

try {
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item -LiteralPath $resolvedExecutable -Destination (Join-Path $stage 'mcp-sync.exe')
    Copy-Item -LiteralPath 'LICENSE', 'README.md', 'Cargo.lock' -Destination $stage
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $archivePath
    Write-Output $archivePath
}
finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}
