param(
    [Parameter(Mandatory = $true)]
    [string]$Executable
)

$ErrorActionPreference = 'Stop'

$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$signature = Get-AuthenticodeSignature -LiteralPath $resolvedExecutable

if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
    throw 'Windows release executable does not have a valid Authenticode signature.'
}
if ($null -eq $signature.SignerCertificate) {
    throw 'Windows release executable does not have a signing certificate.'
}
if ($null -eq $signature.TimeStamperCertificate) {
    throw 'Windows release executable does not have a timestamp certificate.'
}
