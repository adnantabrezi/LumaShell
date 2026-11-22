param(
    [string]$BinaryPath = (Join-Path $PSScriptRoot "..\target\release\rust_shell.exe"),
    [switch]$EnableStartup,
    [switch]$EnableWatchdog,
    [switch]$StartNow,
    [switch]$ReplaceExplorerShell
)

$ErrorActionPreference = "Stop"
$installRoot = Join-Path $env:LOCALAPPDATA "LumaShell"
$installedBinary = Join-Path $installRoot "LumaShell.exe"
$installedWatchdog = Join-Path $installRoot "watchdog.ps1"
$installedRecovery = Join-Path $installRoot "restore-explorer.ps1"
$shellBackup = Join-Path $installRoot "explorer-shell.backup.json"
$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$winlogonKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Winlogon"

if (-not (Test-Path $BinaryPath)) {
    throw "Binary not found: $BinaryPath. Build with 'cargo build --release' first."
}
$manifest = Join-Path (Split-Path -Parent $BinaryPath) "SHA256SUMS.txt"
if (Test-Path $manifest) {
    $expected = (Select-String -Path $manifest -Pattern "LumaShell\.exe" | Select-Object -First 1).Line
    if ([string]::IsNullOrWhiteSpace($expected)) {
        throw "SHA-256 manifest does not contain LumaShell.exe."
    }
    $expectedHash = ($expected -split '\s+')[0]
    $actualHash = (Get-FileHash -Path $BinaryPath -Algorithm SHA256).Hash
    if ($actualHash -ne $expectedHash) {
        throw "SHA-256 verification failed for $BinaryPath."
    }
}
if ($ReplaceExplorerShell -and (-not $EnableStartup -or -not $EnableWatchdog)) {
    throw "-ReplaceExplorerShell requires both -EnableStartup and -EnableWatchdog for rollback safety."
}

New-Item -ItemType Directory -Force -Path $installRoot | Out-Null
$stagedBinary = "$installedBinary.new"
$previousBinary = "$installedBinary.previous"
Copy-Item -Force $BinaryPath $stagedBinary
if (Test-Path $installedBinary) {
    Copy-Item -Force $installedBinary $previousBinary
}
try {
    Move-Item -Force $stagedBinary $installedBinary
} catch {
    if (Test-Path $previousBinary) {
        Copy-Item -Force $previousBinary $installedBinary
    }
    Remove-Item $stagedBinary -Force -ErrorAction SilentlyContinue
    throw "Atomic LumaShell binary replacement failed: $($_.Exception.Message)"
}
$sourceHash = (Get-FileHash -Path $BinaryPath -Algorithm SHA256).Hash
$installedHash = (Get-FileHash -Path $installedBinary -Algorithm SHA256).Hash
if ($sourceHash -ne $installedHash) {
    if (Test-Path $previousBinary) {
        Copy-Item -Force $previousBinary $installedBinary
    }
    throw "Installed binary hash does not match the requested build."
}
Copy-Item -Force (Join-Path $PSScriptRoot "watchdog.ps1") $installedWatchdog
Copy-Item -Force (Join-Path $PSScriptRoot "restore-explorer.ps1") $installedRecovery

if (-not (Test-Path $shellBackup)) {
    $shellValue = (Get-ItemProperty -Path $winlogonKey -Name "Shell" -ErrorAction SilentlyContinue).Shell
    if ([string]::IsNullOrWhiteSpace($shellValue)) { $shellValue = "explorer.exe" }
    [pscustomobject]@{
        Version = 1
        Key = $winlogonKey
        Shell = $shellValue
        CreatedUtc = [DateTime]::UtcNow.ToString("o")
    } | ConvertTo-Json | Set-Content -Path $shellBackup -Encoding UTF8
}

if ($ReplaceExplorerShell) {
    New-Item -Path $winlogonKey -Force | Out-Null
    $shellCommand = "powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$installedWatchdog`" -BinaryPath `"$installedBinary`" -RecoveryScriptPath `"$installedRecovery`" -ShellBackupPath `"$shellBackup`" -RestoreShellOnExit"
    Set-ItemProperty -Path $winlogonKey -Name "Shell" -Value $shellCommand -Type String
}

if (($EnableStartup -or $EnableWatchdog) -and -not $ReplaceExplorerShell) {
    New-Item -Path $runKey -Force | Out-Null
    if ($EnableWatchdog) {
        $command = "powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$installedWatchdog`" -BinaryPath `"$installedBinary`" -RecoveryScriptPath `"$installedRecovery`" -ShellBackupPath `"$shellBackup`""
    } else {
        $command = "`"$installedBinary`""
    }
    New-ItemProperty -Path $runKey -Name "LumaShell" -Value $command -PropertyType String -Force | Out-Null
}
if ($ReplaceExplorerShell) {
    Remove-ItemProperty -Path $runKey -Name "LumaShell" -ErrorAction SilentlyContinue
}

Write-Host "Installed Luma Shell to $installRoot"
if ($EnableStartup) { Write-Host "Startup enabled for the current user." }
if ($EnableWatchdog) { Write-Host "Crash recovery watchdog enabled." }
if ($ReplaceExplorerShell) { Write-Host "Explorer shell replacement enabled with rollback backup at $shellBackup" }
if ($StartNow) {
    if ($ReplaceExplorerShell) {
        Start-Process -FilePath "powershell.exe" -ArgumentList @(
            "-NoProfile", "-WindowStyle", "Hidden", "-ExecutionPolicy", "Bypass",
            "-File", $installedWatchdog, "-BinaryPath", $installedBinary,
            "-RecoveryScriptPath", $installedRecovery, "-ShellBackupPath", $shellBackup,
            "-RestoreShellOnExit"
        )
    } else {
        Start-Process $installedBinary
    }
}
