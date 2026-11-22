param([switch]$KeepSettings)

$ErrorActionPreference = "Stop"
$installRoot = Join-Path $env:LOCALAPPDATA "LumaShell"
$recovery = Join-Path $installRoot "restore-explorer.ps1"
$backup = Join-Path $installRoot "explorer-shell.backup.json"
$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$winlogonKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Winlogon"

Remove-ItemProperty -Path $runKey -Name "LumaShell" -ErrorAction SilentlyContinue
$currentShell = (Get-ItemProperty -Path $winlogonKey -Name "Shell" -ErrorAction SilentlyContinue).Shell
if ((-not [string]::IsNullOrWhiteSpace($currentShell)) -and
    ($currentShell -match "watchdog\.ps1") -and (Test-Path $recovery)) {
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $recovery -ShellBackupPath $backup
}
if (Test-Path $installRoot) {
    if ($KeepSettings) {
        Remove-Item (Join-Path $installRoot "LumaShell.exe") -Force -ErrorAction SilentlyContinue
    } else {
        Remove-Item $installRoot -Recurse -Force
    }
}

Write-Host "Luma Shell startup registration removed; Explorer recovery was applied only when Luma owned the shell command."
