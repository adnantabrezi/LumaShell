Stop-Process -Name rust_shell, LumaShell -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 400

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class KeyFixer3 {
    [DllImport("user32.dll")]
    public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    public static void Clear() {
        keybd_event(0x5B, 0, 2, UIntPtr.Zero);
        keybd_event(0x5C, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        keybd_event(0x10, 0, 2, UIntPtr.Zero);
    }
}
"@
[KeyFixer3]::Clear()

Start-Process -FilePath "D:\project\target\release\rust_shell.exe" -WorkingDirectory "D:\project"
Start-Sleep -Milliseconds 1200
[KeyFixer3]::Clear()
Write-Output "Luma shell started cleanly."
