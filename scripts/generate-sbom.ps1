param(
    [Parameter(Mandatory = $true)]
    [string]$Archive,

    [Parameter(Mandatory = $true)]
    [string]$Output
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$mcpSyncSyftVersion = '1.50.0'
$mcpSyncAssetManifest = if (-not [string]::IsNullOrWhiteSpace($env:MCP_SYNC_SYFT_ASSET_MANIFEST)) {
    $env:MCP_SYNC_SYFT_ASSET_MANIFEST
} else {
    Join-Path $PSScriptRoot 'syft-assets.txt'
}

if (-not (Test-Path -LiteralPath $Archive -PathType Leaf)) {
    throw 'SBOM input archive is missing.'
}
if (-not (Test-Path -LiteralPath $mcpSyncAssetManifest -PathType Leaf)) {
    throw 'Syft asset manifest is missing.'
}

$mcpSyncArchitecture = if (-not [string]::IsNullOrWhiteSpace($env:MCP_SYNC_SYFT_HOST_ARCHITECTURE)) {
    $env:MCP_SYNC_SYFT_HOST_ARCHITECTURE
} else {
    [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
}
switch -Regex ($mcpSyncArchitecture) {
    '^(ARM64|arm64|aarch64)$' { $mcpSyncArchitecture = 'arm64'; break }
    '^(X64|x64|amd64|x86_64)$' { $mcpSyncArchitecture = 'amd64'; break }
    default { throw 'Unsupported Syft host architecture.' }
}

$mcpSyncRecords = @(
    Get-Content -LiteralPath $mcpSyncAssetManifest |
        Where-Object { $_ -notmatch '^\s*(#|$)' } |
        ForEach-Object {
            $fields = $_ -split '\s+'
            if ($fields.Count -ne 4) {
                throw 'Syft asset manifest contains an invalid record.'
            }
            [pscustomobject]@{
                HostOperatingSystem = $fields[0]
                HostArchitecture = $fields[1]
                Asset = $fields[2]
                Sha256 = $fields[3]
            }
        } |
        Where-Object {
            $_.HostOperatingSystem -eq 'windows' -and
            $_.HostArchitecture -eq $mcpSyncArchitecture
        }
)
if ($mcpSyncRecords.Count -ne 1) {
    throw 'Syft asset manifest does not contain exactly one Windows host mapping.'
}
$mcpSyncRecord = $mcpSyncRecords[0]
if ($mcpSyncRecord.Asset -notmatch "^syft_$([regex]::Escape($mcpSyncSyftVersion))_windows_(amd64|arm64)\.zip$") {
    throw 'Syft asset manifest contains an invalid Windows asset name.'
}
if ($mcpSyncRecord.Sha256 -notmatch '^[0-9a-f]{64}$') {
    throw 'Syft asset manifest contains an invalid SHA-256 digest.'
}

$mcpSyncResolvedArchive = (Resolve-Path -LiteralPath $Archive).Path
$mcpSyncOutputParent = Split-Path -Parent $Output
if ([string]::IsNullOrWhiteSpace($mcpSyncOutputParent)) {
    $mcpSyncOutputParent = (Get-Location).Path
}
$mcpSyncOutputDirectory = [IO.Path]::GetFullPath($mcpSyncOutputParent)
New-Item -ItemType Directory -Force -Path $mcpSyncOutputDirectory | Out-Null
$mcpSyncResolvedOutput = Join-Path $mcpSyncOutputDirectory (Split-Path -Leaf $Output)
$mcpSyncTempParent = if (-not [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)) {
    [IO.Path]::GetFullPath($env:RUNNER_TEMP)
} else {
    [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
}
$mcpSyncStage = Join-Path $mcpSyncTempParent ("mcp-sync-syft-" + [guid]::NewGuid().ToString('N'))
$mcpSyncDownload = Join-Path $mcpSyncStage $mcpSyncRecord.Asset
$mcpSyncOutputTemp = Join-Path $mcpSyncOutputDirectory (".mcp-sync-sbom." + [guid]::NewGuid().ToString('N'))

try {
    New-Item -ItemType Directory -Path $mcpSyncStage | Out-Null
    $mcpSyncDownloadBase = if (-not [string]::IsNullOrWhiteSpace($env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL)) {
        $env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL.TrimEnd('/')
    } else {
        "https://github.com/anchore/syft/releases/download/v$mcpSyncSyftVersion"
    }
    $mcpSyncDownloadUri = [uri]"$mcpSyncDownloadBase/$($mcpSyncRecord.Asset)"
    if (-not $mcpSyncDownloadUri.IsAbsoluteUri -or
        $mcpSyncDownloadUri.Scheme -cne [uri]::UriSchemeHttps) {
        throw 'Syft download URI must use HTTPS.'
    }

    $mcpSyncMaximumAttempts = 5
    for ($mcpSyncAttempt = 1; $mcpSyncAttempt -le $mcpSyncMaximumAttempts; $mcpSyncAttempt++) {
        try {
            Invoke-WebRequest `
                -Uri $mcpSyncDownloadUri `
                -OutFile $mcpSyncDownload `
                -MaximumRedirection 5 `
                -TimeoutSec 120
            break
        }
        catch {
            $mcpSyncResponseProperty = $_.Exception.PSObject.Properties['Response']
            $mcpSyncStatusCode = if ($null -ne $mcpSyncResponseProperty -and
                $null -ne $mcpSyncResponseProperty.Value) {
                [int]$mcpSyncResponseProperty.Value.StatusCode
            } else {
                0
            }
            if ($mcpSyncStatusCode -ne 0) {
                $mcpSyncTransient = $mcpSyncStatusCode -in @(408, 429, 500, 502, 503, 504)
            } else {
                $mcpSyncTransient = $false
                $mcpSyncTransportException = $_.Exception
                while ($null -ne $mcpSyncTransportException) {
                    if ($mcpSyncTransportException -is [System.Net.Http.HttpRequestException] -or
                        $mcpSyncTransportException -is [System.TimeoutException] -or
                        $mcpSyncTransportException -is [System.OperationCanceledException] -or
                        $mcpSyncTransportException -is [System.Net.Sockets.SocketException]) {
                        $mcpSyncTransient = $true
                        break
                    }
                    $mcpSyncTransportException = $mcpSyncTransportException.InnerException
                }
            }
            if (-not $mcpSyncTransient -or $mcpSyncAttempt -eq $mcpSyncMaximumAttempts) {
                throw
            }
            $mcpSyncDelaySeconds = [math]::Pow(2, $mcpSyncAttempt - 1)
            Write-Warning "Transient Syft download failure; retrying attempt $($mcpSyncAttempt + 1) of $mcpSyncMaximumAttempts."
            Start-Sleep -Seconds $mcpSyncDelaySeconds
        }
    }

    $mcpSyncActualSha256 = (Get-FileHash -LiteralPath $mcpSyncDownload -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($mcpSyncActualSha256 -cne $mcpSyncRecord.Sha256) {
        throw 'Downloaded Syft asset failed SHA-256 verification.'
    }

    Expand-Archive -LiteralPath $mcpSyncDownload -DestinationPath $mcpSyncStage
    $mcpSyncSyft = Join-Path $mcpSyncStage 'syft.exe'
    if (-not (Test-Path -LiteralPath $mcpSyncSyft -PathType Leaf)) {
        throw 'Verified Syft archive did not contain an executable.'
    }
    $mcpSyncReportedVersion = (& $mcpSyncSyft version | Out-String)
    if ($LASTEXITCODE -ne 0 -or $mcpSyncReportedVersion -notmatch "Version:\s+$([regex]::Escape($mcpSyncSyftVersion))(\s|$)") {
        throw 'Syft executable version does not match the pinned release.'
    }

    $env:SYFT_CHECK_FOR_APP_UPDATE = 'false'
    & $mcpSyncSyft scan "file:$mcpSyncResolvedArchive" --output "spdx-json=$mcpSyncOutputTemp"
    if ($LASTEXITCODE -ne 0) {
        throw 'Syft SPDX JSON generation failed.'
    }
    if (-not (Test-Path -LiteralPath $mcpSyncOutputTemp -PathType Leaf) -or
        (Get-Item -LiteralPath $mcpSyncOutputTemp).Length -eq 0) {
        throw 'Syft did not produce an SPDX JSON document.'
    }
    Move-Item -LiteralPath $mcpSyncOutputTemp -Destination $mcpSyncResolvedOutput -Force
}
finally {
    if (Test-Path -LiteralPath $mcpSyncStage -PathType Container) {
        Remove-Item -LiteralPath $mcpSyncStage -Recurse -Force
    }
    if (Test-Path -LiteralPath $mcpSyncOutputTemp -PathType Leaf) {
        Remove-Item -LiteralPath $mcpSyncOutputTemp -Force
    }
}
