# AGENTS.md

## Project

Luma Shell is a native Windows 11 shell written in Rust. It uses the `windows`
crate for Win32, Shell, GDI, DWM, COM, audio, Wi-Fi, Bluetooth, and registry
APIs.

The executable entry point is `src/main.rs`. The shell implementation is under
`src/shell/`. Platform-independent configuration and formatting logic should
stay independent of Win32 types whenever practical.

## Primary modularity goal

Keep implementation changes modular. Do not add more unrelated behavior to
`src/shell/mod.rs`. It may remain the application composition/root module, but
new Windows features, UI surfaces, system services, or stateful subsystems
must get their own module.

Do not split code merely to reduce file size. Split at ownership and responsibility
boundaries. Prefer several cohesive modules over moving code without moving the
state and invariants that belong to it.

## Intended module layout

Use modules along these responsibility lines:

- `mod.rs`: application lifecycle, module composition, top-level state wiring,
  and the public `run()` entry point.
- `config.rs`: persisted configuration, defaults, validation, parsing, and
  settings file I/O.
- `audio.rs`: Core Audio endpoints, sessions, and callbacks.
- `launcher.rs`: application indexing, launcher query, favorites, and recent
  launches.
- `shell_integration.rs`: Explorer recovery, shell registration, process
  ownership, and shell-policy operations.
- `window.rs` or `windows.rs`: HWND lifecycle, window classes, message loops,
  monitor enumeration, DPI, and shared Win32 window helpers.
- `panel.rs`: top panel window, layout, hit testing, painting, and applet
  dispatch.
- `dock.rs`: dock window, launcher layout, drag/reorder, hover, indicators,
  keyboard navigation, and window activation.
- `desktop.rs`: desktop surface, wallpaper, and desktop context menu.
- `settings.rs`: settings window, pages, controls, layout, and persistence
  actions.
- `popups.rs` or separate popup modules: launcher, switcher, network, Bluetooth,
  audio mixer, tray, and power menus.
- `snipping.rs`: screenshot selection, capture, bitmap conversion, clipboard,
  and PNG output.
- `platform/*.rs`: focused adapters for registry, power, network, Bluetooth,
  process/window enumeration, virtual desktop, and notification APIs.

Names are guidance. A module must own its public types and behavior rather than
depend on a flat collection of unrelated globals.

## State ownership

Before implementing a feature, identify who owns it:

- Window handle
- User configuration
- Cached system snapshot
- Display/monitor geometry
- Native COM callback or endpoint
- User-selection state
- Native bitmap/icon/resource lifetime

Do not introduce new `static mut` variables. For existing state, minimize its
scope and convert groups of related globals into a struct owned by the owning
window or subsystem.

Suitable state mechanisms include:

- `OnceLock`, `LazyLock`, or `Mutex` for genuinely process-wide immutable or
  synchronized state.
- A window procedure's per-module `state` struct for UI state.
- `Arc<Mutex<T>>` only when callbacks and UI threads genuinely share state.
- `Send`, `Sync`, and thread-affinity boundaries must be stated in code and
  validated; do not assume HWND or COM objects are thread-safe.

Global variables may remain temporarily when extracted code still depends on
them, but the extraction should move ownership toward a parameter or state
struct instead of adding another global.

## Win32 and unsafe boundaries

Unsafe Rust is expected for Win32 FFI and COM, but it must be bounded and
reviewable.

- Keep unsafe operations near the API boundary.
- Prefer small safe Rust wrappers around native calls.
- Document pointer, handle, callback, thread-affinity, and lifetime requirements
  in a brief safety comment.
- Do not call `std::mem::forget` to compensate for leaked native resources.
- Destroy/close native resources using the matching Windows API:
  handles, devices, DIB/DC objects, brushes, pens, regions, hooks, timers,
  thumbnails, clipboard handles, and COM references.
- Validate pointers and native handles before dereferencing or publishing them.
- Preserve Win32 error handling and logging. Do not silently replace native
  failures with plausible defaults.
- Convert native snapshot types into owned Rust types such as `String`, `Vec`,
  and plain structs before passing them across unrelated modules.
- Do not expose raw pointers or native handles as a module's general-purpose
  API unless native callbacks or window procedures require them.
- Keep `unsafe extern "system"` callback bodies small and delegate into safe
  module functions.

Do not remove `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
or broaden Windows-only APIs to unsupported targets without cfg guards.

## Public API rules

Expose only the operations needed by other modules.

Prefer:

```rust
pub(crate) struct NetworkSnapshot { ... }
pub(crate) fn network_snapshot() -> NetworkSnapshot
```

Over module APIs that expose mutable global internals or a list of independent
raw setters.

Avoid `use super::*` in new modules. Import explicit types and APIs. Existing
modules may be migrated gradually, but new code must not increase cross-module
coupling.

Use `pub(crate)` for project-internal modules, types, and helpers. Reserve
`pub` for a genuinely external API or the executable entry point.

## Feature implementation

For UI or system-feature work:

1. Search for an existing module/helper before implementing a duplicate.
2. Identify all native resources and all mutable state.
3. Add the module or extract the smallest cohesive subsystem.
4. Keep Win32 message routing and pure Rust decisions separate.
5. Update configuration defaults, parsing, persistence, and settings UI
   together when a feature becomes configurable.
6. Add focused tests for parsing, validation, hit testing, layout calculations,
   state transitions, and snapshot transformations.
7. Keep native-interaction tests platform-gated or behind narrow traits/adapters.
8. Preserve shell-replacement, Explorer recovery, and watchdog behavior unless
   the request explicitly changes those policies.

Do not put paint code and process/network/registry I/O in the same helper unless
the Windows API or callback genuinely requires it.

## Extraction rules

When moving code out of `mod.rs`:

- Move the related types and state with the functions.
- Replace global lookups with constructor parameters or an owned state struct
  when the move is complete.
- Preserve message constants and control IDs in the module that owns the window
  or controls that use them.
- Preserve existing log messages when moving diagnostic behavior.
- Delete dead code only if it is unquestionably part of the move; do not mix
  extraction with unrelated cleanup.
- Update `mod.rs` to compose the module, not re-export a large flattened API.
- Build and test the smallest affected target before broad validation.

Keep large extracted modules under roughly 1,500 lines where practical. This is
not a hard limit; cohesive state may justify a larger module, but unrelated UI
surfaces, services, and window procedures should not be combined.

## Testing and validation

Run the narrowest relevant command first:

```powershell
cargo fmt --check
cargo check --target x86_64-pc-windows-msvc
cargo test --target x86_64-pc-windows-msvc
cargo clippy --target x86_64-pc-windows-msvc -- -D warnings
cargo build --release --target x86_64-pc-windows-msvc
```

The project may also use the GNU Windows target in existing build artifacts.
Prefer the active toolchain/target unless changing cross-compilation behavior.

For changes that can run in an executable or shell process:

```powershell
.\target\release\rust_shell.exe
```

Smoke-test native behavior in a real Windows session. In particular verify:

- Primary and secondary monitor layout/DPI changes
- AppBar registration and fullscreen behavior
- Keyboard hooks and registered hotkeys
- Shell cleanup and Explorer restore behavior
- Config reload/save/backup behavior
- Popup focus, Escape, and keyboard navigation
- Native resource cleanup on close, timer, hook, and callback teardown

Use existing PowerShell installer scripts for packaging or startup changes.
Do not assume a build success proves that native window or shell behavior works.

## Change discipline

- Make the smallest complete change.
- Do not reformat unrelated code.
- Do not alter `settings.ini` schema or config version casually.
- Do not change Winlogon shell registration or watchdog recovery behavior unless
  the task explicitly requires it.
- Preserve existing behavior unless an intentional behavior change is documented.
- Add or update tests only for behavior introduced or fixed by the change.
- Surface native API failures explicitly through `Result` or existing logging.
