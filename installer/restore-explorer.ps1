param(
    [Parameter(Mandatory = $true)][string]$ShellBackupPath,
    [switch]$RequireLumaOwnership
)

$ErrorActionPreference = "Stop"
$winlogonKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Winlogon"
$shell = "explorer.exe"
$currentShell = (Get-ItemProperty -Path $winlogonKey -Name "Shell" -ErrorAction SilentlyContinue).Shell
if ($RequireLumaOwnership -and
    ([string]::IsNullOrWhiteSpace($currentShell) -or $currentShell -notmatch "watchdog\.ps1")) {
    throw "Explorer shell restore refused because Luma does not own the current shell entry."
}

if (Test-Path $ShellBackupPath) {
    $backup = Get-Content -Raw -Path $ShellBackupPath | ConvertFrom-Json
    if (-not [string]::IsNullOrWhiteSpace($backup.Shell)) {
        $shell = [string]$backup.Shell
    }
}

New-Item -Path $winlogonKey -Force | Out-Null
Set-ItemProperty -Path $winlogonKey -Name "Shell" -Value $shell -Type String
Start-Process "explorer.exe"
Write-Host "Explorer shell restored."
