param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedVersion
)

$ErrorActionPreference = 'Stop'

$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$smokeRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("mcp-sync-release-smoke-" + [guid]::NewGuid().ToString('N'))
$smokeHome = Join-Path $smokeRoot 'home'
$smokeLocal = Join-Path $smokeRoot 'local'
$smokeRoaming = Join-Path $smokeRoot 'roaming'
$smokeXdg = Join-Path $smokeRoot 'xdg'

function Invoke-McpSync {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $resolvedExecutable
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.Environment.Clear()
    $startInfo.Environment['USERPROFILE'] = $smokeHome
    $startInfo.Environment['HOME'] = $smokeHome
    $startInfo.Environment['LOCALAPPDATA'] = $smokeLocal
    $startInfo.Environment['APPDATA'] = $smokeRoaming
    $startInfo.Environment['XDG_CONFIG_HOME'] = $smokeXdg
    $startInfo.Environment['PATH'] = $env:PATH

    foreach ($argument in $Arguments) {
        $startInfo.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw 'Could not start the installed mcp-sync executable.'
    }

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    if ($process.ExitCode -ne 0) {
        throw "Installed mcp-sync command failed with exit code $($process.ExitCode): $stderr"
    }
    if (-not [string]::IsNullOrEmpty($stderr)) {
        throw "Installed mcp-sync command wrote unexpected stderr: $stderr"
    }

    return $stdout.TrimEnd("`r", "`n")
}

try {
    New-Item -ItemType Directory -Path $smokeHome, $smokeLocal, $smokeRoaming, $smokeXdg | Out-Null

    $versionOutput = Invoke-McpSync '--version'
    if ($versionOutput -ne "mcp-sync $ExpectedVersion") {
        throw 'Installed executable reported an unexpected version.'
    }

    Invoke-McpSync 'init' | Out-Null
    Invoke-McpSync 'add' 'release-smoke' '--command' 'release-smoke-server' '--arg=--stdio' '--env' 'SMOKE_TOKEN=synthetic-release-value' | Out-Null
    Invoke-McpSync 'sync' '--dry-run' | Out-Null
    Invoke-McpSync 'sync' | Out-Null
    Invoke-McpSync 'restore' 'canonical' '--dry-run' | Out-Null
    Invoke-McpSync 'restore' 'canonical' | Out-Null

    $restoredList = Invoke-McpSync 'list'
    if ($restoredList.Contains('release-smoke')) {
        throw 'First restore did not recover the empty canonical generation.'
    }

    Invoke-McpSync 'restore' 'canonical' | Out-Null
    $currentList = Invoke-McpSync 'list'
    if (-not $currentList.Contains('release-smoke')) {
        throw 'Second restore did not recover the newer canonical generation.'
    }

    Invoke-McpSync 'sync' '--dry-run' | Out-Null
}
finally {
    if (Test-Path -LiteralPath $smokeRoot) {
        Remove-Item -LiteralPath $smokeRoot -Recurse -Force
    }
}
