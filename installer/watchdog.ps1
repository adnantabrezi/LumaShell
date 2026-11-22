param(
    [Parameter(Mandatory = $true)][string]$BinaryPath,
    [Parameter(Mandatory = $true)][string]$RecoveryScriptPath,
    [Parameter(Mandatory = $true)][string]$ShellBackupPath,
    [switch]$RestoreShellOnExit,
    [int]$MaxCrashes = 3,
    [int]$CrashWindowSeconds = 120
)

$ErrorActionPreference = "Stop"
$stateRoot = Split-Path -Parent $ShellBackupPath
$logPath = Join-Path $stateRoot "watchdog.log"
New-Item -ItemType Directory -Force -Path $stateRoot | Out-Null
function Write-WatchdogLog([string]$Message) {
    Add-Content -Path $logPath -Value "$(Get-Date -Format o) $Message"
    if ((Get-Item $logPath).Length -gt 1048576) {
        $rotated = "$logPath.1"
        Remove-Item $rotated -Force -ErrorAction SilentlyContinue
        Move-Item $logPath $rotated -Force
    }
}
function Restore-Shell {
    if (-not $RestoreShellOnExit) { return }
    try {
        if (-not (Test-Path $RecoveryScriptPath)) {
            throw "Recovery script is unavailable"
        }
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $RecoveryScriptPath -ShellBackupPath $ShellBackupPath
        Write-WatchdogLog "Recovery script completed"
        return
    } catch {
        Write-WatchdogLog "Recovery script failed: $($_.Exception.Message)"
    }
    try {
        $winlogonKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Winlogon"
        New-Item -Path $winlogonKey -Force | Out-Null
        Set-ItemProperty -Path $winlogonKey -Name "Shell" -Value "explorer.exe" -Type String
        Start-Process "explorer.exe"
        Write-WatchdogLog "Direct Explorer fallback completed"
    } catch {
        Write-WatchdogLog "Direct Explorer fallback failed: $($_.Exception.Message)"
    }
}
if (-not (Test-Path $BinaryPath)) {
    Write-WatchdogLog "Binary missing: $BinaryPath"
    Restore-Shell
    exit 1
}
if ($RestoreShellOnExit -and -not (Test-Path $RecoveryScriptPath)) {
    Write-WatchdogLog "Recovery script missing: $RecoveryScriptPath"
}
$crashes = 0
$windowStart = Get-Date
Write-WatchdogLog "Watchdog started; shell mode=$RestoreShellOnExit"
while ($true) {
    $process = Start-Process -FilePath $BinaryPath -WorkingDirectory (Split-Path -Parent $BinaryPath) -PassThru
    $process.WaitForExit()
    $exitCode = $process.ExitCode
    Write-WatchdogLog "Luma exited with code $exitCode"
    if ($exitCode -eq 0) {
        if ($RestoreShellOnExit) {
            Write-WatchdogLog "Normal exit; restoring Explorer"
            Restore-Shell
        }
        break
    }
    $now = Get-Date
    if (($now - $windowStart).TotalSeconds -gt $CrashWindowSeconds) {
        $windowStart = $now
        $crashes = 0
    }
    $crashes++
    Write-WatchdogLog "Crash $crashes/$MaxCrashes in current window"
    if ($crashes -ge $MaxCrashes) {
        if ($RestoreShellOnExit) {
            Write-WatchdogLog "Crash threshold reached; restoring Explorer"
            Restore-Shell
        }
        exit 1
    }
    Start-Sleep -Seconds 2
}
