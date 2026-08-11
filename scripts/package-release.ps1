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
$mcpSyncRepositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

if ($Version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
    throw 'Release version must be a canonical stable semantic version.'
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
$archiveTemp = Join-Path $resolvedOutput (".$archiveName." + [guid]::NewGuid().ToString('N') + '.tmp.zip')

try {
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item -LiteralPath $resolvedExecutable -Destination (Join-Path $stage 'mcp-sync.exe')
    Copy-Item -LiteralPath `
        (Join-Path $mcpSyncRepositoryRoot 'LICENSE'), `
        (Join-Path $mcpSyncRepositoryRoot 'README.md'), `
        (Join-Path $mcpSyncRepositoryRoot 'Cargo.lock') `
        -Destination $stage
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $archiveTemp
    Move-Item -LiteralPath $archiveTemp -Destination $archivePath -Force
    Write-Output $archivePath
}
finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    if (Test-Path -LiteralPath $archiveTemp -PathType Leaf) {
        Remove-Item -LiteralPath $archiveTemp -Force
    }
}
