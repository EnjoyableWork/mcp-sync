param(
    [Parameter(Mandatory = $true)]
    [string]$Archive,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion
)

$ErrorActionPreference = 'Stop'

$resolvedArchive = (Resolve-Path -LiteralPath $Archive).Path
$extractRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("mcp-sync-release-archive-" + [guid]::NewGuid().ToString('N'))

try {
    New-Item -ItemType Directory -Path $extractRoot | Out-Null
    Expand-Archive -LiteralPath $resolvedArchive -DestinationPath $extractRoot

    foreach ($requiredFile in @('mcp-sync.exe', 'LICENSE', 'README.md', 'Cargo.lock')) {
        if (-not (Test-Path -LiteralPath (Join-Path $extractRoot $requiredFile) -PathType Leaf)) {
            throw "Release archive is missing $requiredFile."
        }
    }

    & (Join-Path $PSScriptRoot 'smoke-installed.ps1') `
        -Executable (Join-Path $extractRoot 'mcp-sync.exe') `
        -ExpectedVersion $ExpectedVersion
}
finally {
    if (Test-Path -LiteralPath $extractRoot) {
        Remove-Item -LiteralPath $extractRoot -Recurse -Force
    }
}
