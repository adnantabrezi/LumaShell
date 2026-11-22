param(
    [string]$BinaryPath = (Join-Path $PSScriptRoot "..\target\release\rust_shell.exe"),
    [string]$OutputPath = (Join-Path $PSScriptRoot "..\dist"),
    [string]$CertificateThumbprint = ""
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BinaryPath)) {
    throw "Release binary not found: $BinaryPath. Build with 'cargo build --release' first."
}

$stage = Join-Path $OutputPath "LumaShell"
if (Test-Path $stage) {
    Remove-Item $stage -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item -Force $BinaryPath (Join-Path $stage "LumaShell.exe")
Copy-Item -Force (Join-Path $PSScriptRoot "install.ps1") $stage
Copy-Item -Force (Join-Path $PSScriptRoot "uninstall.ps1") $stage
Copy-Item -Force (Join-Path $PSScriptRoot "watchdog.ps1") $stage
Copy-Item -Force (Join-Path $PSScriptRoot "restore-explorer.ps1") $stage

if (-not [string]::IsNullOrWhiteSpace($CertificateThumbprint)) {
    $certificate = Get-ChildItem "Cert:\CurrentUser\My\$CertificateThumbprint" -ErrorAction Stop | Select-Object -First 1
    if ($null -eq $certificate) {
        throw "Signing certificate not found: $CertificateThumbprint"
    }
    Get-ChildItem $stage -File | ForEach-Object {
        $signature = Set-AuthenticodeSignature -FilePath $_.FullName -Certificate $certificate
        if ($signature.Status -ne "Valid") {
            throw "Authenticode signing failed for $($_.Name): $($signature.Status)"
        }
    }
}

$hashes = Get-ChildItem $stage -File | Get-FileHash -Algorithm SHA256 |
    ForEach-Object { "$($_.Hash)  $($_.Path | Split-Path -Leaf)" }
$hashes | Set-Content (Join-Path $stage "SHA256SUMS.txt") -Encoding ASCII

Write-Host "Release staged at $stage"
if ([string]::IsNullOrWhiteSpace($CertificateThumbprint)) {
    Write-Host "Authenticode signing was not requested; provide -CertificateThumbprint for signed output."
}
