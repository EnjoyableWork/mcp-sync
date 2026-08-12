$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$mcpSyncRepositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$mcpSyncGenerator = Join-Path $PSScriptRoot 'generate-sbom.ps1'
$mcpSyncManifest = Join-Path $PSScriptRoot 'syft-assets.txt'
$mcpSyncManifestText = Get-Content -LiteralPath $mcpSyncManifest -Raw
$mcpSyncGeneratorText = Get-Content -LiteralPath $mcpSyncGenerator -Raw

class McpSyncSyntheticHttpException : System.Exception {
    [object]$Response

    McpSyncSyntheticHttpException([int]$StatusCode) : base("Synthetic HTTP $StatusCode response") {
        $this.Response = [pscustomobject]@{ StatusCode = $StatusCode }
    }
}

foreach ($mcpSyncContract in @(
    "`$mcpSyncMaximumAttempts = 5",
    '408, 429, 500, 502, 503, 504',
    'HttpRequestException',
    'System.TimeoutException',
    'System.OperationCanceledException',
    'System.Net.Sockets.SocketException',
    'if (-not $mcpSyncTransient -or $mcpSyncAttempt -eq $mcpSyncMaximumAttempts)',
    'Get-FileHash -LiteralPath $mcpSyncDownload -Algorithm SHA256',
    "`$env:SYFT_CHECK_FOR_APP_UPDATE = 'false'"
)) {
    if (-not $mcpSyncGeneratorText.Contains($mcpSyncContract)) {
        throw "Windows Syft acquisition contract is missing: $mcpSyncContract"
    }
}

foreach ($mcpSyncRecord in @(
    'windows arm64 syft_1.50.0_windows_arm64.zip 5eb435eb8750737d12e66f5a145975b4027adf20076b518079af38b2148d55a5',
    'windows amd64 syft_1.50.0_windows_amd64.zip 815ee6973ec5dff6a671d7f41b0e78835a8c45b91d5a39f4743ea1cee833d3be'
)) {
    if (-not $mcpSyncManifestText.Contains($mcpSyncRecord)) {
        throw "Syft asset manifest is missing: $mcpSyncRecord"
    }
}

$mcpSyncTempRoot = Join-Path ([IO.Path]::GetTempPath()) ("mcp-sync-syft-policy-" + [guid]::NewGuid().ToString('N'))
$mcpSyncPreviousArchitecture = $env:MCP_SYNC_SYFT_HOST_ARCHITECTURE
$mcpSyncPreviousManifest = $env:MCP_SYNC_SYFT_ASSET_MANIFEST
$mcpSyncPreviousBaseUrl = $env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL
try {
    New-Item -ItemType Directory -Path $mcpSyncTempRoot | Out-Null
    $mcpSyncInput = Join-Path $mcpSyncTempRoot 'input.zip'
    Set-Content -LiteralPath $mcpSyncInput -Value 'synthetic release archive'

    $env:MCP_SYNC_SYFT_HOST_ARCHITECTURE = 'riscv64'
    $mcpSyncRejected = $false
    try {
        & $mcpSyncGenerator -Archive $mcpSyncInput -Output (Join-Path $mcpSyncTempRoot 'output.spdx.json')
    }
    catch {
        $mcpSyncRejected = $_.Exception.Message -eq 'Unsupported Syft host architecture.'
    }
    if (-not $mcpSyncRejected) {
        throw 'Windows Syft generator did not reject an unsupported architecture before download.'
    }

    $mcpSyncFixtureManifest = Join-Path $mcpSyncTempRoot 'syft-assets.txt'
    Set-Content -LiteralPath $mcpSyncFixtureManifest -Value @'
windows arm64 syft_1.50.0_windows_arm64.zip 0000000000000000000000000000000000000000000000000000000000000000
'@
    $env:MCP_SYNC_SYFT_HOST_ARCHITECTURE = 'arm64'
    $env:MCP_SYNC_SYFT_ASSET_MANIFEST = $mcpSyncFixtureManifest
    $env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL = 'https://example.invalid/syft'

    $global:mcpSyncSyntheticDownloadAttempts = 0
    $global:mcpSyncSyntheticSleepCount = 0
    function global:Invoke-WebRequest {
        $global:mcpSyncSyntheticDownloadAttempts += 1
        throw [McpSyncSyntheticHttpException]::new(503)
    }
    function global:Start-Sleep {
        $global:mcpSyncSyntheticSleepCount += 1
    }
    try {
        & $mcpSyncGenerator -Archive $mcpSyncInput -Output (Join-Path $mcpSyncTempRoot 'transient.spdx.json')
        throw 'Windows Syft generator accepted exhausted transient acquisition failures.'
    }
    catch {
        if ($_.Exception.Message -eq 'Windows Syft generator accepted exhausted transient acquisition failures.') {
            throw
        }
    }
    if ($global:mcpSyncSyntheticDownloadAttempts -ne 5 -or $global:mcpSyncSyntheticSleepCount -ne 4) {
        throw 'Windows Syft generator did not enforce the exact transient retry bound.'
    }

    $global:mcpSyncSyntheticDownloadAttempts = 0
    $global:mcpSyncSyntheticSleepCount = 0
    function global:Invoke-WebRequest {
        $global:mcpSyncSyntheticDownloadAttempts += 1
        throw [McpSyncSyntheticHttpException]::new(404)
    }
    try {
        & $mcpSyncGenerator -Archive $mcpSyncInput -Output (Join-Path $mcpSyncTempRoot 'permanent.spdx.json')
        throw 'Windows Syft generator accepted a permanent acquisition failure.'
    }
    catch {
        if ($_.Exception.Message -eq 'Windows Syft generator accepted a permanent acquisition failure.') {
            throw
        }
    }
    if ($global:mcpSyncSyntheticDownloadAttempts -ne 1 -or $global:mcpSyncSyntheticSleepCount -ne 0) {
        throw 'Windows Syft generator retried a permanent HTTP response.'
    }

    $env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL = 'http://example.invalid/syft'
    $global:mcpSyncSyntheticDownloadAttempts = 0
    $mcpSyncRejected = $false
    try {
        & $mcpSyncGenerator -Archive $mcpSyncInput -Output (Join-Path $mcpSyncTempRoot 'insecure.spdx.json')
    }
    catch {
        $mcpSyncRejected = $_.Exception.Message -eq 'Syft download URI must use HTTPS.'
    }
    if (-not $mcpSyncRejected -or $global:mcpSyncSyntheticDownloadAttempts -ne 0) {
        throw 'Windows Syft generator accepted an insecure download URI.'
    }
}
finally {
    $env:MCP_SYNC_SYFT_HOST_ARCHITECTURE = $mcpSyncPreviousArchitecture
    $env:MCP_SYNC_SYFT_ASSET_MANIFEST = $mcpSyncPreviousManifest
    $env:MCP_SYNC_SYFT_DOWNLOAD_BASE_URL = $mcpSyncPreviousBaseUrl
    Remove-Item -Path Function:\global:Invoke-WebRequest -ErrorAction SilentlyContinue
    Remove-Item -Path Function:\global:Start-Sleep -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $mcpSyncTempRoot -PathType Container) {
        Remove-Item -LiteralPath $mcpSyncTempRoot -Recurse -Force
    }
}

Write-Output 'Windows Syft SBOM generation policy tests passed.'
