# Luma Shell

Luma is a native Windows 11 shell surface with a macOS/COSMIC-inspired floating dock, a compact top panel, modular applets, and a settings GUI.

## Current product surface

- Floating dock with Home, Files, Terminal, Browser, Mail, and Settings launchers.
- High-resolution native icons for dock launchers resolved from system binaries (`imageres.dll`, `shell32.dll`, `explorer.exe`, `wt.exe`, `msedge.exe`, `SystemSettings.exe`, user profile) via `SHGetFileInfoW` and `ExtractIconExW`, rendered with `DrawIconEx`.
- Modern squircle colored rounded tiles with Segoe Fluent Icons and MDL2 Assets vector glyph fallbacks.
- Complete Windows shell replacement integration:
  - Registers as the official shell window via `SetShellWindow` and listens to `RegisterShellHookWindow` (`WM_SHELLHOOK`) for instant dock window lifecycle updates.
  - Low-level keyboard hook (`WH_KEYBOARD_LL`) enables solo Windows key tap to toggle the Application Launcher without interfering with Windows combination shortcuts.
  - Dedicated desktop background window (`LumaDesktop`) pinned at `HWND_BOTTOM` rendering Windows wallpaper (`SPI_GETDESKWALLPAPER`) or gradient, with full right-click context menu (Terminal, Files, Launcher, Switcher, Display Settings, Personalization, Settings, Power actions, Restore Explorer).
  - Integrated Power & Session menu in top panel with Lock, Sign Out, Sleep, Restart, and Shut Down (`LockWorkStation`, `ExitWindowsEx`).
  - In-GUI shell replacement toggle in Settings Recovery page ("Set Luma as Default Shell" / "Restore Windows Explorer") managing `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Winlogon\Shell`.
- Top panel with Luma identity pill (`\u{E707}`), Activities affordance pill (`\u{E7C4}`), modular applets, and dedicated Settings (`\u{E713}`) and Power (`\u{E7E8}`) buttons.
- Applet modules for clock (`\u{E823}`), network (`\u{E701}`/`\u{E839}`), volume/mute with dynamic audio level glyphs (`\u{E74F}`/`\u{E993}`/`\u{E994}`/`\u{E767}`), Bluetooth-radio presence (`\u{E702}`), battery with dynamic level glyphs (`\u{E945}`/`\u{E850}`..`\u{E859}`), notifications (`\u{EA8F}`), and media transport session (`\u{E8D6}`).
- `Win + Alt + Space` toggles the dock without hijacking Windows' input-language shortcut.
- `Win + Alt + L` opens the launcher and focuses its search field for keyboard-only use; registration failures are logged instead of overriding another shortcut.
- Settings window for dark/light theme, desktop wallpaper & context menu toggle, dock position, applet visibility, 12/24-hour time, `none`/`short`/`long` date formats, and a per-user “Start with Windows” toggle.
- Settings is organized into General, Dock, Applets, and Recovery pages with a vertical sidebar and native keyboard-focusable navigation buttons.
- Dock customization includes auto-hide, small/medium/large icon sizes, magnification strength, transparency, spacing, and x/y position offsets; values are range-validated and persisted.
- Dock launchers can be drag-reordered and the order persists in Settings as `home,files,terminal,browser,mail,settings`.
- Settings persist to `%APPDATA%\LumaShell\settings.ini`, with validation, schema version `7`, atomic replacement, and a `.bak` recovery copy; unknown future versions fall back to safe defaults.
- Dock positions: left, center, and right.
- Applications panel indexes `.lnk`, `.url`, and `.exe` entries from both user and system Start Menu folders, exposes curated Wi-Fi/sound/display/Bluetooth/notification/personalization/update Settings destinations, and filters them live as you type.
- The launcher index refreshes periodically while the shell is running, so newly installed or removed Start Menu applications appear without restarting Luma.
- Launcher queries of two or more characters also perform a bounded filename search across Desktop, Documents, and Downloads.
- Launcher results support Up/Down/Home/End/Page Up/Page Down/Enter/Escape keyboard navigation, keep a capped recent-launch list in `%APPDATA%\LumaShell\recent.txt`, and support persistent favorites in `%APPDATA%\LumaShell\favorites.txt` via Ctrl+D or right-click.
- Launcher rows resolve native Windows icons for applications, Settings destinations, and matching files; favorites retain a visible star marker.
- Dock indicators are derived from visible top-level windows and matched to common Explorer, terminal, browser, mail, and Settings processes instead of fixed placeholder dots.
- The foreground window's matching dock launcher receives a distinct active highlight, separate from hover feedback.
- Clicking a launcher’s running indicator restores and activates that launcher’s matching window.
- Dock indicators refresh twice per second as windows appear, close, minimize, or restore.
- Clicking a running dock launcher activates or restores its existing window; clicking the active window minimizes it instead of launching a duplicate.
- The dock supports keyboard-only navigation with Left/Right/Home/End, Enter, and Escape after opening it with `Win + Alt + Space`; the launcher and window switcher also support Home/End and page jumps.
- `Win + Tab` opens a window switcher listing visible application windows and activating the selected one.
- `Win + Alt + Left/Right` forwards the native `Win + Ctrl + Left/Right` shortcuts for switching Windows virtual desktops without taking over the system Alt-Tab chord.
- `Win + Alt + Shift + Left/Right` forwards the native `Win + Ctrl + Shift + Left/Right` shortcuts to move the active window between adjacent virtual desktops; registration failures are logged rather than overriding another shortcut.
- Window enumeration is virtual-desktop aware through `IVirtualDesktopManager`, so switcher entries and running indicators are scoped to the current Windows workspace.
- The switcher renders a live DWM thumbnail for the selected window and supports Up/Down/Enter/Escape selection.
- Windows-only runtime boundary with a clean non-Windows fallback.
- AppBar registration reserves the top panel area, single-instance protection prevents duplicate shells and brings the existing panel forward, and rounded-corner styling is applied through DWM.
- Empty panel regions return `HTTRANSPARENT`, allowing maximized applications to receive clicks while launcher, gear, and applet regions remain interactive.
- Battery percentage/AC state and active network-interface state are read from Windows APIs rather than being hard-coded glyphs; connected Wi-Fi now reports the current SSID and signal quality through WLAN API data, and Wi-Fi adapters are identified separately from wired interfaces.
- Network adapter/WLAN queries are snapshot-cached for one second so the live panel remains responsive without repeatedly opening WLAN handles during every repaint.
- Volume level and mute state are read from the Windows default render endpoint through `IAudioEndpointVolume`; the panel also counts active Core Audio sessions, and the process opts into per-monitor-V2 DPI awareness for mixed-DPI layouts.
- Dock and panel dimensions are scaled against each monitor's DPI and repositioned after DPI, display, and work-area changes.
- Settings, Launcher, and Switcher apply Windows' suggested `WM_DPICHANGED` rectangles when moved between mixed-DPI monitors.
- Launcher and Settings popups center on the monitor that initiated the action rather than always opening on the primary display.
- Secondary monitors receive their own panel and dock windows, with per-monitor AppBar registration and cleanup when displays change.
- Secondary AppBar position callbacks reposition only their own monitor dock; the primary callback alone rebuilds the secondary set, avoiding callback-time window destruction.
- Panel controls use each monitor panel's client geometry, so launcher and Settings hit zones remain correct on secondary displays.
- Auto-hide checks every monitor's dock and work-area edge, so secondary-monitor docks reveal correctly when the pointer reaches that display's bottom edge.
- AppBar callbacks keep work-area changes synchronized and hide docks during fullscreen AppBar notifications; the primary AppBar reservation is retained and reapplied across DPI/display changes, including monitors with negative virtual-desktop coordinates.
- Settings includes a manual Restore Explorer action for safe recovery during development.
- Manual Restore Explorer uses an ownership guard and refuses to overwrite another shell's Winlogon entry; watchdog recovery remains explicitly authorized for shell-replacement mode.
- Windows session shutdown/logoff is handled explicitly so the normal cleanup path removes AppBars, secondary monitor windows, hotkeys, and DWM thumbnails.
- Settings' Recovery page includes an Open diagnostics action for the `%APPDATA%\LumaShell\luma.log` file.
- The Recovery page also exposes the watchdog log at `%LOCALAPPDATA%\LumaShell\watchdog.log`.
- Settings' Recovery page can atomically restore the last validated `settings.ini.bak` snapshot, roll back a failed commit, and reload it immediately.
- Applet order is editable in Settings with a validated comma-separated list such as `clock,network,volume,bluetooth,battery,notifications,media`.
- Applet zones are editable with validated assignments such as `clock:left,network:center,volume:right`; zone rendering and hit testing follow the persisted assignments.
- Dock, launcher, and switcher windows use Windows 11 DWM rounded-corner/Mica styling with hover feedback on dock launchers.
- Dock hover magnification uses a separate 16 ms easing timer, while app/window state refresh remains on a slower timer.
- Shell surfaces also use native layered alpha for a translucent finish.
- DWM immersive dark mode is synchronized with the Settings theme across primary and secondary shell windows.
- Windows theme, high-contrast, and system-setting notifications invalidate and restyle every shell surface, including launcher, switcher, and audio mixer popups.
- Panel applets are actionable: network and battery indicators open their matching Windows Settings pages, while the volume applet opens a native Core Audio mixer with executable-labeled master and per-session mute/volume controls; it falls back to Sound Settings when the master endpoint is unavailable.
- The audio mixer is keyboard-dismissable with Escape and closes when focus leaves the popup.
- The audio mixer supports Home (master mute), Up/Down/Left/Right/Enter session control, selected-row highlighting, and Win32 accessibility focus events for application sessions.
- Applet hit testing follows the configured panel order, including Bluetooth and media actions, so reordering cannot redirect a click to the wrong control.
- When a media session is active, the media applet shows play/pause state and clicking its panel zone invokes the real Windows play/pause transport command.
- Notification state detects quiet time/presentation mode through `SHQueryUserNotificationState` and opens Windows notification settings when clicked.
- Custom panel and dock rendering detects Windows High Contrast mode and switches to an accessible black/white/yellow palette.
- High Contrast mode also forces opaque shell surfaces so user-selected transparency cannot reduce contrast.
- Dock magnification honors the Windows client-area animation preference and snaps without motion when reduced-motion behavior is enabled.
- Native Settings and Launcher buttons/edit fields participate explicitly in Windows Tab navigation.
- Custom dock keyboard selection emits standard Win32 accessibility focus events so assistive technology can track the selected launcher.
- Launcher result selection also emits accessibility focus events for keyboard and mouse selection.
- The launcher search edit uses a native Windows cue banner (`Search applications`) for discoverability and assistive-technology metadata.
- Window-switcher selection emits accessibility focus events while navigating visible windows.

## Modular architecture

Applets implement the small `Applet` interface in `src/main.rs`: they report whether they are enabled and render themselves into the panel. Adding a new applet means adding one implementation plus a config flag and settings control. The panel and dock are independent Win32 windows, so they can evolve separately into richer services such as workspace indicators, notifications, media controls, or a launcher.

## Build and run

```powershell
cargo build --release
.\target\release\rust_shell.exe
```

For an opt-in per-user install and startup registration, run PowerShell from the project root:

```powershell
.\installer\install.ps1 -EnableStartup -StartNow
```

Add automatic restart after a crash with `-EnableWatchdog` in normal startup mode. The watchdog stops when Luma exits normally and does not change the Explorer shell registration in that mode. Shell-replacement mode adds an explicit recovery flag so only that mode restores Explorer on exit or repeated crashes; if the recovery script itself is unavailable, a direct Explorer fallback is attempted. Watchdog startup, exit codes, crash counts, and recovery actions are recorded in `%LOCALAPPDATA%\LumaShell\watchdog.log` with one rotated file.

To stage a reproducible release with SHA-256 checksums, run `.\installer\package.ps1`. The exact `dist\LumaShell` staging directory is rebuilt on each run, and `install.ps1` verifies `LumaShell.exe` when that manifest is present. Pass `-CertificateThumbprint <thumbprint>` to Authenticode-sign the staged executable and PowerShell scripts with a certificate in the current user's certificate store; requested signing fails closed if any signature is not valid.

The installer stages the new executable before replacing the live copy, keeps a `.previous` rollback binary, and verifies the installed hash before continuing.

The Windows CI workflow repeats formatting/build/check/test/Clippy gates, parses all installer scripts, verifies the watchdog/recovery contract, stages and smoke-checks the package manifest, and uploads it as a verifiable artifact.

For an explicit, opt-in shell replacement with rollback protection, use `-ReplaceExplorerShell -EnableStartup -EnableWatchdog`. This stores the previous Winlogon shell in `%LOCALAPPDATA%\LumaShell\explorer-shell.backup.json`, runs the watchdog as the shell command, and restores Explorer after repeated crashes. Uninstall or the Settings recovery action restores the saved value.

Remove the startup registration with `.\installer\uninstall.ps1`; it also invokes the Explorer recovery script before removing the install files.

The shell does not replace `explorer.exe` automatically. Keep it opt-in during development. The current implementation still needs richer workspace switching, true system Alt-Tab integration, UI Automation metadata for custom surfaces, certificate-backed release signing, and an end-to-end Windows GUI recovery test before broad distribution.
