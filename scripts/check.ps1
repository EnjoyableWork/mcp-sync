$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$mcpSyncScriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$mcpSyncRepositoryRoot = [IO.Path]::GetFullPath((Join-Path $mcpSyncScriptDirectory ".."))
$mcpSyncCallerUserRoot = $env:USERPROFILE
if ([string]::IsNullOrWhiteSpace($mcpSyncCallerUserRoot)) {
    throw "USERPROFILE must identify the caller Rust toolchain root"
}
$mcpSyncCargoHome = if ([string]::IsNullOrWhiteSpace($env:CARGO_HOME)) {
    Join-Path $mcpSyncCallerUserRoot ".cargo"
} else {
    $env:CARGO_HOME
}
$mcpSyncRustupHome = if ([string]::IsNullOrWhiteSpace($env:RUSTUP_HOME)) {
    Join-Path $mcpSyncCallerUserRoot ".rustup"
} else {
    $env:RUSTUP_HOME
}
$mcpSyncTempParent = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$mcpSyncSyntheticRoot = Join-Path $mcpSyncTempParent ("mcp-sync-quality-" + [guid]::NewGuid().ToString("N"))
$mcpSyncSyntheticPrefix = Join-Path $mcpSyncTempParent "mcp-sync-quality-"
$mcpSyncSyntheticUserRoot = Join-Path $mcpSyncSyntheticRoot "user"
$mcpSyncLocationPushed = $false

function Assert-CargoGate {
    param([Parameter(Mandatory = $true)][string]$Name)

    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

try {
    $directories = @(
        (Join-Path $mcpSyncSyntheticUserRoot ".cache"),
        (Join-Path $mcpSyncSyntheticUserRoot ".config"),
        (Join-Path $mcpSyncSyntheticUserRoot ".local/share"),
        (Join-Path $mcpSyncSyntheticUserRoot ".local/state"),
        (Join-Path $mcpSyncSyntheticUserRoot "AppData/Local"),
        (Join-Path $mcpSyncSyntheticUserRoot "AppData/Roaming"),
        (Join-Path $mcpSyncSyntheticUserRoot "Library/Application Support"),
        (Join-Path $mcpSyncSyntheticRoot "runtime"),
        (Join-Path $mcpSyncSyntheticRoot "tmp"),
        (Join-Path $mcpSyncSyntheticRoot "xdg-config-dirs")
    )
    New-Item -ItemType Directory -Force -Path $directories | Out-Null

    $env:APPDATA = Join-Path $mcpSyncSyntheticUserRoot "AppData/Roaming"
    $env:CARGO_HOME = $mcpSyncCargoHome
    $env:CARGO_INCREMENTAL = "0"
    $env:CARGO_TERM_COLOR = "never"
    $env:CFFIXED_USER_HOME = $mcpSyncSyntheticUserRoot
    $env:HOME = $mcpSyncSyntheticUserRoot
    $env:LANG = "C"
    $env:LC_ALL = "C"
    $env:LOCALAPPDATA = Join-Path $mcpSyncSyntheticUserRoot "AppData/Local"
    $env:MCP_SYNC_TEST_HOME = $mcpSyncSyntheticUserRoot
    $env:MCP_SYNC_TEST_MODE = "1"
    $env:MCP_SYNC_TEST_ROOT = $mcpSyncSyntheticRoot
    $env:NO_COLOR = "1"
    $env:RUSTUP_HOME = $mcpSyncRustupHome
    $env:TEMP = Join-Path $mcpSyncSyntheticRoot "tmp"
    $env:TMP = Join-Path $mcpSyncSyntheticRoot "tmp"
    $env:TMPDIR = Join-Path $mcpSyncSyntheticRoot "tmp"
    $env:TZ = "UTC"
    $env:USERPROFILE = $mcpSyncSyntheticUserRoot
    $env:XDG_CACHE_HOME = Join-Path $mcpSyncSyntheticUserRoot ".cache"
    $env:XDG_CONFIG_DIRS = Join-Path $mcpSyncSyntheticRoot "xdg-config-dirs"
    $env:XDG_CONFIG_HOME = Join-Path $mcpSyncSyntheticUserRoot ".config"
    $env:XDG_DATA_HOME = Join-Path $mcpSyncSyntheticUserRoot ".local/share"
    $env:XDG_RUNTIME_DIR = Join-Path $mcpSyncSyntheticRoot "runtime"
    $env:XDG_STATE_HOME = Join-Path $mcpSyncSyntheticUserRoot ".local/state"

    Push-Location $mcpSyncRepositoryRoot
    $mcpSyncLocationPushed = $true
    Write-Output "Running quality gates with disposable Windows user configuration roots."

    & cargo fmt --all -- --check
    Assert-CargoGate "Formatting"
    & cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    Assert-CargoGate "Clippy"
    & cargo test --workspace --all-targets --all-features --locked
    Assert-CargoGate "Tests"

    Write-Output "Formatting, Clippy, and tests passed through the synthetic Windows home."
} finally {
    if ($mcpSyncLocationPushed) {
        Pop-Location
    }

    $mcpSyncResolvedRoot = [IO.Path]::GetFullPath($mcpSyncSyntheticRoot)
    if (!$mcpSyncResolvedRoot.StartsWith($mcpSyncSyntheticPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove unexpected quality-gate path: $mcpSyncResolvedRoot"
    }
    if (Test-Path -LiteralPath $mcpSyncResolvedRoot -PathType Container) {
        Remove-Item -LiteralPath $mcpSyncResolvedRoot -Recurse -Force
    }
}
