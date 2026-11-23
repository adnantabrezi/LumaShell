//! GlazeWM v3-inspired COSMIC Tiling & Floating Window Manager for Windows 11.
//!
//! Design & Architecture:
//! - Pure Rust recursive `TilingNode` (`Split` / `Window`) layout solver with proportional
//!   child weights, inner/outer gaps, single-child flattening, directional swap, split
//!   flipping, proportional edge-resize clamping, and cubic-out interpolation.
//! - Multi-monitor `trees_by_monitor` state (each HMONITOR manages its own `SplitContainer`
//!   tree independently) + Windows 11 `IVirtualDesktopManager` current-workspace filtering.
//! - GlazeWM-grade Windows 11 window eligibility filtering (excludes own PID, shell HWNDs,
//!   owned dialogs/popups, tool/non-resizable windows, fullscreen games/apps, cloaked UWP
//!   windows, and Windows shell host classes).
//! - Non-blocking ~100Hz smooth animation engine driven by `SetTimer(panel, ID_TILING_ANIM_TIMER, 10)`
//!   with seamless mid-flight retargeting and atomic `BeginDeferWindowPos` / `DeferWindowPos` /
//!   `EndDeferWindowPos` frame commits (zero UI thread blocking).
//! - GlazeWM-style `DWMWA_EXTENDED_FRAME_BOUNDS` invisible-border compensation (`RectDelta`),
//!   COSMIC Active Focus Hint via `DWMWA_BORDER_COLOR` (34), mouse drag-to-swap & drag-to-resize,
//!   and complete `UnhookWinEvent` + border-color restoration on shutdown.
//! - State ownership: process-wide synchronized `OnceLock<Mutex<TilingState>>` (zero `static mut`).

use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DWMWINDOWATTRIBUTE, DwmGetWindowAttribute,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Shell::{IVirtualDesktopManager, VirtualDesktopManager};
use windows::Win32::UI::WindowsAndMessaging::{
    BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, EnumWindows, GA_ROOT, GW_OWNER,
    GWL_EXSTYLE, GWL_STYLE, GetAncestor, GetClassNameW, GetForegroundWindow, GetWindow,
    GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow,
    IsWindowVisible, IsZoomed, KillTimer, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER,
    SetForegroundWindow, SetTimer, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_CHILD,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_THICKFRAME,
};

/// Timer ID registered on `PANEL` for non-blocking ~100Hz smooth tiling animations.
pub(crate) const ID_TILING_ANIM_TIMER: usize = 702;
/// One-shot timer ID registered on `PANEL` to sync tiles cleanly after virtual desktop switches.
pub(crate) const ID_TILING_WORKSPACE_SYNC_TIMER: usize = 703;

/// Win32 WinEvent constants used by GlazeWM-style window lifecycle hooks.
const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const EVENT_SYSTEM_MOVESIZEEND: u32 = 0x000B;
const EVENT_SYSTEM_MINIMIZESTART: u32 = 0x0016;
const EVENT_SYSTEM_MINIMIZEEND: u32 = 0x0017;
const EVENT_OBJECT_DESTROY: u32 = 0x8001;
const EVENT_OBJECT_SHOW: u32 = 0x8002;
const EVENT_OBJECT_HIDE: u32 = 0x8003;
const OBJID_WINDOW: i32 = 0;

/// Windows 11 DWM border color attribute (`DWMWA_BORDER_COLOR = 34`) for COSMIC Active Hint.
const DWMWA_BORDER_COLOR: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(34);
const DWMWA_COLOR_DEFAULT: u32 = 0xFFFF_FFFF;

/// Known Windows system host classes that must never be tiled.
const IGNORED_WINDOW_CLASSES: &[&str] = &[
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Progman",
    "WorkerW",
    "Windows.UI.Core.CoreWindow",
    "Xaml_WindowedPopupClass",
    "ForegroundStaging",
    "MultitaskingViewFrame",
    "NotifyIconOverflowWindow",
    "TaskListThumbnailWnd",
    "TopLevelWindowForOverflowXamlIsland",
    "Windows.UI.Input.InputSite.WindowClass",
    "IME",
    "MSCTFIME UI",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TilingDirection {
    Horizontal,
    Vertical,
}

impl TilingDirection {
    pub(crate) fn flip(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TileRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl TileRect {
    pub(crate) fn from_win32(r: RECT) -> Self {
        Self {
            x: r.left,
            y: r.top,
            width: (r.right - r.left).max(0),
            height: (r.bottom - r.top).max(0),
        }
    }

    pub(crate) fn center(self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub(crate) fn contains_point(self, px: i32, py: i32) -> bool {
        px >= self.x
            && px <= self.x + self.width
            && py >= self.y
            && py <= self.y + self.height
    }

    pub(crate) fn with_border_delta(self, delta: RectDelta) -> Self {
        Self {
            x: self.x - delta.left,
            y: self.y - delta.top,
            width: (self.width + delta.left + delta.right).max(80),
            height: (self.height + delta.top + delta.bottom).max(60),
        }
    }

    pub(crate) fn approx_eq(self, other: Self, tolerance: i32) -> bool {
        (self.x - other.x).abs() <= tolerance
            && (self.y - other.y).abs() <= tolerance
            && (self.width - other.width).abs() <= tolerance
            && (self.height - other.height).abs() <= tolerance
    }
}

/// GlazeWM-style compensation for Windows 11 invisible drop-shadow / resize frame margins.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RectDelta {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TilingNode {
    Window {
        hwnd: isize,
        size: f32,
    },
    Split {
        direction: TilingDirection,
        size: f32,
        children: Vec<TilingNode>,
    },
}

impl TilingNode {
    pub(crate) fn size(&self) -> f32 {
        match self {
            Self::Window { size, .. } | Self::Split { size, .. } => *size,
        }
    }

    fn set_size(&mut self, next: f32) {
        match self {
            Self::Window { size, .. } | Self::Split { size, .. } => *size = next,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TilingSettings {
    pub(crate) enabled: bool,
    pub(crate) direction: TilingDirection,
    pub(crate) inner_gap: i32,
    pub(crate) outer_gap: i32,
    pub(crate) active_hint: bool,
    pub(crate) accent_rgb: u32,
    pub(crate) animation_ms: u32,
    pub(crate) dock_auto_hide: bool,
}

impl Default for TilingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            direction: TilingDirection::Horizontal,
            inner_gap: 10,
            outer_gap: 12,
            active_hint: true,
            accent_rgb: 0x38bdf8,
            animation_ms: 240,
            dock_auto_hide: false,
        }
    }
}

/// Read-only snapshot for the COSMIC Tiling Popup and Panel UI.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TilingSnapshot {
    pub(crate) enabled: bool,
    pub(crate) direction: TilingDirection,
    pub(crate) inner_gap: i32,
    pub(crate) outer_gap: i32,
    pub(crate) active_hint: bool,
    pub(crate) animation_ms: u32,
    pub(crate) tiled_count: usize,
    pub(crate) floating_count: usize,
    pub(crate) focused_is_floating: bool,
}

struct AnimItem {
    hwnd: isize,
    from: TileRect,
    to: TileRect,
    current: TileRect,
}

struct ActiveAnimation {
    started_at: Instant,
    duration_ms: u32,
    items: Vec<AnimItem>,
}

/// Owned tiling subsystem state protected by a `Mutex` inside `OnceLock`.
/// Handles are stored as `isize` so the struct is cleanly `Send`; Win32 calls always
/// validate handles with `IsWindow` on the UI thread before use.
struct TilingState {
    settings: TilingSettings,
    panel_hwnd: isize,
    /// Separate GlazeWM `SplitContainer` tree per HMONITOR so multi-monitor setups tile cleanly.
    trees_by_monitor: HashMap<isize, TilingNode>,
    /// Last computed visual target rects (uncompensated screen coordinates) for drag-to-swap / resize.
    last_tiled_rects: HashMap<isize, TileRect>,
    /// Windows explicitly floated by the user (preserved as COSMIC floating exceptions).
    floating_windows: HashSet<isize>,
    /// Pre-tiling floating rectangles saved so toggling Floating Mode restores natural positions.
    saved_floating_rects: HashMap<isize, TileRect>,
    /// All windows whose `DWMWA_BORDER_COLOR` we modified so shutdown restores default borders cleanly.
    bordered_windows: HashSet<isize>,
    focused_hwnd: Option<isize>,
    shell_hwnds: HashSet<isize>,
    event_hooks: Vec<isize>,
    animation: Option<ActiveAnimation>,
}

impl TilingState {
    fn new() -> Self {
        Self {
            settings: TilingSettings::default(),
            panel_hwnd: 0,
            trees_by_monitor: HashMap::new(),
            last_tiled_rects: HashMap::new(),
            floating_windows: HashSet::new(),
            saved_floating_rects: HashMap::new(),
            bordered_windows: HashSet::new(),
            focused_hwnd: None,
            shell_hwnds: HashSet::new(),
            event_hooks: Vec::new(),
            animation: None,
        }
    }

    fn find_monitor_of(&self, hwnd: isize) -> Option<isize> {
        self.trees_by_monitor
            .iter()
            .find_map(|(&mon, tree)| tree_contains(Some(tree), hwnd).then_some(mon))
    }

    fn all_tiled_ids(&self) -> Vec<isize> {
        let mut out = Vec::new();
        for tree in self.trees_by_monitor.values() {
            collect_rec(tree, &mut out);
        }
        out
    }

    fn remove_from_all_trees(&mut self, hwnd: isize) -> bool {
        let mut removed = false;
        let keys: Vec<isize> = self.trees_by_monitor.keys().copied().collect();
        for mon in keys {
            if let Some(tree) = self.trees_by_monitor.remove(&mon) {
                if tree_contains(Some(&tree), hwnd) {
                    removed = true;
                    if let Some(next) = remove_window_from_tree(Some(tree), hwnd) {
                        self.trees_by_monitor.insert(mon, next);
                    }
                } else {
                    self.trees_by_monitor.insert(mon, tree);
                }
            }
        }
        self.last_tiled_rects.remove(&hwnd);
        removed
    }
}

static TILING_STATE: OnceLock<Mutex<TilingState>> = OnceLock::new();

fn state() -> &'static Mutex<TilingState> {
    TILING_STATE.get_or_init(|| Mutex::new(TilingState::new()))
}

// ============================================================================
// Pure Rust GlazeWM SplitContainer Tree Operations
// ============================================================================

pub(crate) fn collect_window_ids(node: Option<&TilingNode>) -> Vec<isize> {
    let Some(n) = node else { return Vec::new() };
    let mut out = Vec::new();
    collect_rec(n, &mut out);
    out
}

fn collect_rec(node: &TilingNode, out: &mut Vec<isize>) {
    match node {
        TilingNode::Window { hwnd, .. } => out.push(*hwnd),
        TilingNode::Split { children, .. } => {
            for child in children {
                collect_rec(child, out);
            }
        }
    }
}

pub(crate) fn tree_contains(node: Option<&TilingNode>, target: isize) -> bool {
    let Some(n) = node else { return false };
    match n {
        TilingNode::Window { hwnd, .. } => *hwnd == target,
        TilingNode::Split { children, .. } => {
            children.iter().any(|c| tree_contains(Some(c), target))
        }
    }
}

fn normalize_weights(children: &mut [TilingNode]) {
    if children.is_empty() {
        return;
    }
    let total: f32 = children.iter().map(|c| c.size().max(0.1)).sum();
    if total <= 0.0 {
        for c in children.iter_mut() {
            c.set_size(1.0);
        }
        return;
    }
    let scale = children.len() as f32 / total;
    for c in children.iter_mut() {
        let next = (c.size().max(0.1) * scale).max(0.15);
        c.set_size(next);
    }
}

pub(crate) fn insert_window_into_tree(
    root: Option<TilingNode>,
    new_hwnd: isize,
    focused_hwnd: Option<isize>,
    direction: TilingDirection,
) -> TilingNode {
    if tree_contains(root.as_ref(), new_hwnd) {
        return root.unwrap();
    }
    let leaf = TilingNode::Window {
        hwnd: new_hwnd,
        size: 1.0,
    };
    let Some(mut root_node) = root else {
        return leaf;
    };

    if let TilingNode::Window { hwnd, .. } = root_node {
        return TilingNode::Split {
            direction,
            size: 1.0,
            children: vec![TilingNode::Window { hwnd, size: 1.0 }, leaf],
        };
    }

    let target = focused_hwnd
        .filter(|&id| tree_contains(Some(&root_node), id))
        .or_else(|| collect_window_ids(Some(&root_node)).last().copied());

    let Some(target_id) = target else {
        if let TilingNode::Split { children, .. } = &mut root_node {
            children.push(leaf);
            normalize_weights(children);
        }
        return root_node;
    };

    let _ = insert_recursive(&mut root_node, new_hwnd, target_id, direction);
    root_node
}

fn insert_recursive(
    node: &mut TilingNode,
    new_hwnd: isize,
    target_id: isize,
    direction: TilingDirection,
) -> bool {
    match node {
        TilingNode::Window { .. } => false,
        TilingNode::Split {
            direction: split_dir,
            children,
            ..
        } => {
            if let Some(pos) = children
                .iter()
                .position(|c| matches!(c, TilingNode::Window { hwnd, .. } if *hwnd == target_id))
            {
                if *split_dir == direction {
                    children.insert(
                        pos + 1,
                        TilingNode::Window {
                            hwnd: new_hwnd,
                            size: 1.0,
                        },
                    );
                    normalize_weights(children);
                } else {
                    let old_size = children[pos].size();
                    children[pos] = TilingNode::Split {
                        direction,
                        size: old_size,
                        children: vec![
                            TilingNode::Window {
                                hwnd: target_id,
                                size: 1.0,
                            },
                            TilingNode::Window {
                                hwnd: new_hwnd,
                                size: 1.0,
                            },
                        ],
                    };
                }
                return true;
            }
            for child in children.iter_mut() {
                if insert_recursive(child, new_hwnd, target_id, direction) {
                    return true;
                }
            }
            false
        }
    }
}

pub(crate) fn remove_window_from_tree(
    root: Option<TilingNode>,
    window_id: isize,
) -> Option<TilingNode> {
    let node = root?;
    match node {
        TilingNode::Window { hwnd, .. } => {
            if hwnd == window_id {
                None
            } else {
                Some(node)
            }
        }
        TilingNode::Split {
            direction,
            size,
            children,
        } => {
            let mut kept: Vec<TilingNode> = children
                .into_iter()
                .filter_map(|child| remove_window_from_tree(Some(child), window_id))
                .collect();
            if kept.is_empty() {
                None
            } else if kept.len() == 1 {
                let mut only = kept.remove(0);
                only.set_size(size);
                Some(only)
            } else {
                normalize_weights(&mut kept);
                Some(TilingNode::Split {
                    direction,
                    size,
                    children: kept,
                })
            }
        }
    }
}

pub(crate) fn compute_tiled_layout(
    root: Option<&TilingNode>,
    work_area: TileRect,
    inner_gap: i32,
    outer_gap: i32,
) -> HashMap<isize, TileRect> {
    let mut rects = HashMap::new();
    let Some(node) = root else { return rects };

    let outer = outer_gap.clamp(0, 48);
    let inner = inner_gap.clamp(0, 48);
    let usable = TileRect {
        x: work_area.x + outer,
        y: work_area.y + outer,
        width: (work_area.width - outer * 2).max(160),
        height: (work_area.height - outer * 2).max(140),
    };
    layout_recursive(node, usable, inner, &mut rects);
    rects
}

fn layout_recursive(
    node: &TilingNode,
    bounds: TileRect,
    inner_gap: i32,
    out: &mut HashMap<isize, TileRect>,
) {
    match node {
        TilingNode::Window { hwnd, .. } => {
            out.insert(*hwnd, bounds);
        }
        TilingNode::Split {
            direction,
            children,
            ..
        } => {
            let count = children.len();
            if count == 0 {
                return;
            }
            let total_weight: f32 = children.iter().map(|c| c.size().max(0.1)).sum();
            let total_gap = inner_gap * (count as i32 - 1).max(0);
            match direction {
                TilingDirection::Horizontal => {
                    let avail_w = (bounds.width - total_gap).max(count as i32 * 80);
                    let mut cursor_x = bounds.x;
                    for (idx, child) in children.iter().enumerate() {
                        let is_last = idx + 1 == count;
                        let share = child.size().max(0.1) / total_weight;
                        let child_w = if is_last {
                            (bounds.x + bounds.width - cursor_x).max(80)
                        } else {
                            (avail_w as f32 * share).round() as i32
                        };
                        layout_recursive(
                            child,
                            TileRect {
                                x: cursor_x,
                                y: bounds.y,
                                width: child_w,
                                height: bounds.height,
                            },
                            inner_gap,
                            out,
                        );
                        cursor_x += child_w + inner_gap;
                    }
                }
                TilingDirection::Vertical => {
                    let avail_h = (bounds.height - total_gap).max(count as i32 * 80);
                    let mut cursor_y = bounds.y;
                    for (idx, child) in children.iter().enumerate() {
                        let is_last = idx + 1 == count;
                        let share = child.size().max(0.1) / total_weight;
                        let child_h = if is_last {
                            (bounds.y + bounds.height - cursor_y).max(80)
                        } else {
                            (avail_h as f32 * share).round() as i32
                        };
                        layout_recursive(
                            child,
                            TileRect {
                                x: bounds.x,
                                y: cursor_y,
                                width: bounds.width,
                                height: child_h,
                            },
                            inner_gap,
                            out,
                        );
                        cursor_y += child_h + inner_gap;
                    }
                }
            }
        }
    }
}

pub(crate) fn swap_window_ids(node: &mut TilingNode, a: isize, b: isize) {
    if a == b {
        return;
    }
    match node {
        TilingNode::Window { hwnd, .. } => {
            if *hwnd == a {
                *hwnd = b;
            } else if *hwnd == b {
                *hwnd = a;
            }
        }
        TilingNode::Split { children, .. } => {
            for c in children.iter_mut() {
                swap_window_ids(c, a, b);
            }
        }
    }
}

pub(crate) fn swap_window_with_neighbor(
    root: Option<&mut TilingNode>,
    window_id: isize,
    forward: bool,
) {
    let Some(node) = root else { return };
    let ordered = collect_window_ids(Some(node));
    let Some(idx) = ordered.iter().position(|&id| id == window_id) else {
        return;
    };
    if ordered.len() < 2 {
        return;
    }
    let neighbor_idx = if forward {
        (idx + 1) % ordered.len()
    } else {
        (idx + ordered.len() - 1) % ordered.len()
    };
    let other_id = ordered[neighbor_idx];
    swap_window_ids(node, window_id, other_id);
}

pub(crate) fn toggle_split_direction_at(
    root: Option<&mut TilingNode>,
    window_id: Option<isize>,
) {
    let Some(node) = root else { return };
    if let Some(target) = window_id {
        if flip_parent_recursive(node, target) {
            return;
        }
    }
    if let TilingNode::Split { direction, .. } = node {
        *direction = direction.flip();
    }
}

fn flip_parent_recursive(node: &mut TilingNode, target: isize) -> bool {
    match node {
        TilingNode::Window { .. } => false,
        TilingNode::Split {
            direction,
            children,
            ..
        } => {
            if children
                .iter()
                .any(|c| matches!(c, TilingNode::Window { hwnd, .. } if *hwnd == target))
            {
                *direction = direction.flip();
                return true;
            }
            children
                .iter_mut()
                .any(|c| flip_parent_recursive(c, target))
        }
    }
}

/// Adjusts the proportional weight of `window_id` within its parent `SplitContainer`
/// (clamped safely between 0.35 and 2.5 so no sibling ever collapses).
pub(crate) fn resize_window_in_tree(
    root: Option<&mut TilingNode>,
    window_id: isize,
    delta: f32,
) -> bool {
    let Some(node) = root else { return false };
    resize_recursive(node, window_id, delta)
}

fn resize_recursive(node: &mut TilingNode, target: isize, delta: f32) -> bool {
    match node {
        TilingNode::Window { .. } => false,
        TilingNode::Split { children, .. } => {
            if let Some(pos) = children
                .iter()
                .position(|c| matches!(c, TilingNode::Window { hwnd, .. } if *hwnd == target))
            {
                if children.len() < 2 {
                    return false;
                }
                let current = children[pos].size();
                let next_size = (current + delta).clamp(0.35, 2.5);
                let actual_delta = next_size - current;
                if actual_delta.abs() < 0.001 {
                    return false;
                }
                let others = (children.len() - 1) as f32;
                let other_delta = -actual_delta / others;
                for (idx, child) in children.iter_mut().enumerate() {
                    if idx == pos {
                        child.set_size(next_size);
                    } else {
                        child.set_size((child.size() + other_delta).clamp(0.35, 2.5));
                    }
                }
                normalize_weights(children);
                return true;
            }
            children
                .iter_mut()
                .any(|c| resize_recursive(c, target, delta))
        }
    }
}

pub(crate) fn parent_direction_of(node: &TilingNode, target: isize) -> Option<TilingDirection> {
    match node {
        TilingNode::Window { .. } => None,
        TilingNode::Split {
            direction,
            children,
            ..
        } => {
            if children
                .iter()
                .any(|c| matches!(c, TilingNode::Window { hwnd, .. } if *hwnd == target))
            {
                return Some(*direction);
            }
            children
                .iter()
                .find_map(|c| parent_direction_of(c, target))
        }
    }
}

pub(crate) fn ease_out_cubic(t: f32) -> f32 {
    let c = t.clamp(0.0, 1.0);
    1.0 - (1.0 - c).powi(3)
}

pub(crate) fn lerp_rect(from: TileRect, to: TileRect, t: f32) -> TileRect {
    let k = ease_out_cubic(t);
    TileRect {
        x: (from.x as f32 + (to.x - from.x) as f32 * k).round() as i32,
        y: (from.y as f32 + (to.y - from.y) as f32 * k).round() as i32,
        width: (from.width as f32 + (to.width - from.width) as f32 * k).round() as i32,
        height: (from.height as f32 + (to.height - from.height) as f32 * k).round() as i32,
    }
}

// ============================================================================
// Win32 / DWM Integration Layer (Bounded Unsafe)
// ============================================================================

fn rgb_to_colorref(rgb: u32) -> u32 {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;
    (b << 16) | (g << 8) | r
}

fn compute_dwm_border_delta(hwnd: HWND) -> RectDelta {
    let mut win_rect = RECT::default();
    let mut ext_rect = RECT::default();
    // SAFETY: Caller verified `hwnd` is a valid live window; both stack `RECT` pointers are valid.
    unsafe {
        if GetWindowRect(hwnd, &mut win_rect).is_err() {
            return RectDelta::default();
        }
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut ext_rect as *mut RECT as *mut _,
            size_of::<RECT>() as u32,
        );
        if hr.is_err() {
            return RectDelta::default();
        }
    }
    RectDelta {
        left: (ext_rect.left - win_rect.left).max(0),
        top: (ext_rect.top - win_rect.top).max(0),
        right: (win_rect.right - ext_rect.right).max(0),
        bottom: (win_rect.bottom - ext_rect.bottom).max(0),
    }
}

/// Returns the visible DWM frame bounds (excluding Windows 11's invisible 7px resize shadows),
/// falling back to `GetWindowRect` if DWM cannot report extended bounds.
fn window_visual_rect(hwnd: HWND) -> Option<TileRect> {
    let mut ext_rect = RECT::default();
    // SAFETY: Caller verified `hwnd` is a valid live window; stack `RECT` pointer is valid.
    unsafe {
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut ext_rect as *mut RECT as *mut _,
            size_of::<RECT>() as u32,
        );
        if hr.is_ok() && (ext_rect.right > ext_rect.left) && (ext_rect.bottom > ext_rect.top) {
            return Some(TileRect::from_win32(ext_rect));
        }
        let mut win_rect = RECT::default();
        if GetWindowRect(hwnd, &mut win_rect).is_ok() {
            return Some(TileRect::from_win32(win_rect));
        }
    }
    None
}

fn is_window_cloaked(hwnd: HWND) -> bool {
    let mut cloaked: u32 = 0;
    // SAFETY: `cloaked` is a valid u32 stack buffer matching `DWMWA_CLOAKED` size.
    let hr = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            size_of::<u32>() as u32,
        )
    };
    hr.is_ok() && cloaked != 0
}

fn create_virtual_desktop_manager() -> Option<IVirtualDesktopManager> {
    // SAFETY: COM is initialized on the UI thread before tiling hooks start.
    unsafe { CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL).ok() }
}

fn is_on_current_virtual_desktop_with(
    hwnd: HWND,
    vdm: Option<&IVirtualDesktopManager>,
) -> bool {
    let Some(manager) = vdm else {
        return true;
    };
    // SAFETY: Validated live HWND passed to STA COM `IVirtualDesktopManager`.
    unsafe {
        manager
            .IsWindowOnCurrentVirtualDesktop(hwnd)
            .map(|b| b.as_bool())
            .unwrap_or(true)
    }
}

fn is_fullscreen_on_monitor(hwnd: HWND) -> bool {
    // SAFETY: Read-only Win32 monitor and window rect queries on a validated HWND.
    unsafe {
        let mut wr = RECT::default();
        if GetWindowRect(hwnd, &mut wr).is_err() {
            return false;
        }
        let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(mon, &mut info).as_bool() {
            return false;
        }
        let m = info.rcMonitor;
        wr.left <= m.left && wr.top <= m.top && wr.right >= m.right && wr.bottom >= m.bottom
    }
}

fn window_class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    // SAFETY: `buf` is a valid mutable UTF-16 slice on the stack.
    let len = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    if len == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len.min(buf.len())])
}

/// GlazeWM-style eligibility filter for top-level resizable user application windows.
fn should_manage_window(
    hwnd: HWND,
    shell_hwnds: &HashSet<isize>,
    vdm: Option<&IVirtualDesktopManager>,
) -> bool {
    if hwnd.is_invalid() || shell_hwnds.contains(&(hwnd.0 as isize)) {
        return false;
    }
    // SAFETY: Read-only Win32 state queries on a non-null window handle.
    unsafe {
        if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() {
            return false;
        }
        if GetAncestor(hwnd, GA_ROOT) != hwnd {
            return false;
        }
        // Owned popup/dialog windows (Save As, Find, Splash) float automatically.
        if matches!(GetWindow(hwnd, GW_OWNER), Ok(owner) if !owner.is_invalid()) {
            return false;
        }
        if IsIconic(hwnd).as_bool() || IsZoomed(hwnd).as_bool() {
            return false;
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if (style & WS_CHILD.0) != 0 || (style & WS_THICKFRAME.0) == 0 {
            return false;
        }
        if (ex_style & (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0)) != 0 {
            return false;
        }
    }
    let class = window_class_name(hwnd);
    if IGNORED_WINDOW_CLASSES.iter().any(|&ignored| ignored == class) {
        return false;
    }
    if is_window_cloaked(hwnd) || is_fullscreen_on_monitor(hwnd) {
        return false;
    }
    is_on_current_virtual_desktop_with(hwnd, vdm)
}

fn monitor_work_area(monitor_raw: isize, dock_auto_hide: bool, panel_hwnd: isize) -> TileRect {
    use windows::Win32::Graphics::Gdi::HMONITOR;
    let mon = HMONITOR(monitor_raw as *mut _);
    // SAFETY: `MONITORINFO` has `cbSize` properly initialized before calling `GetMonitorInfoW`.
    unsafe {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(mon, &mut info).as_bool() {
            let mut work = TileRect::from_win32(info.rcWork);
            if !dock_auto_hide {
                let dpi = if panel_hwnd != 0 {
                    let d = GetDpiForWindow(HWND(panel_hwnd as *mut _));
                    if d == 0 { 96 } else { d }
                } else {
                    96
                };
                let dock_clearance = (76 * dpi as i32 + 48) / 96;
                work.height = (work.height - dock_clearance).max(240);
            }
            return work;
        }
    }
    TileRect {
        x: 0,
        y: 38,
        width: 1920,
        height: 964,
    }
}

fn apply_active_hint_borders(
    windows: &[isize],
    focused: Option<isize>,
    active_hint: bool,
    accent_rgb: u32,
) {
    let accent_colorref = rgb_to_colorref(accent_rgb);
    for &raw in windows {
        let hwnd = HWND(raw as *mut _);
        // SAFETY: Validated non-null live window; `color` is a stack `u32`.
        unsafe {
            if !IsWindow(hwnd).as_bool() {
                continue;
            }
            let color = if active_hint && focused == Some(raw) {
                accent_colorref
            } else {
                DWMWA_COLOR_DEFAULT
            };
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                &color as *const u32 as *const _,
                size_of::<u32>() as u32,
            );
        }
    }
}

/// Commits a single frame of window rectangles atomically using `BeginDeferWindowPos` /
/// `DeferWindowPos` / `EndDeferWindowPos` (zero flickering, zero thread sleep).
fn commit_window_frame(frame: &[(isize, TileRect)]) {
    if frame.is_empty() {
        return;
    }
    // SAFETY: `BeginDeferWindowPos` handle is committed on the same thread via `EndDeferWindowPos`.
    // `HWND::default()` is ignored because `SWP_NOZORDER` is set.
    unsafe {
        let Ok(mut hdwp) = BeginDeferWindowPos(frame.len() as i32) else {
            return;
        };
        for &(raw, r) in frame {
            let hwnd = HWND(raw as *mut _);
            if !IsWindow(hwnd).as_bool() || IsIconic(hwnd).as_bool() || IsZoomed(hwnd).as_bool() {
                continue;
            }
            if let Ok(next_hdwp) = DeferWindowPos(
                hdwp,
                hwnd,
                HWND::default(),
                r.x,
                r.y,
                r.width,
                r.height,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
            ) {
                hdwp = next_hdwp;
            }
        }
        let _ = EndDeferWindowPos(hdwp);
    }
}

/// Schedules a non-blocking smooth animation from each window's current rect (or in-flight
/// animated rect for seamless mid-flight retargeting) to its DWM-compensated target rect.
fn schedule_animated_placement(targets: &HashMap<isize, TileRect>, animation_ms: u32) {
    if targets.is_empty() {
        return;
    }

    let (panel_hwnd, existing_in_flight, preserved_others) = {
        let guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let mut cur_map = HashMap::new();
        let mut others = Vec::new();
        if let Some(anim) = &guard.animation {
            for item in &anim.items {
                cur_map.insert(item.hwnd, item.current);
                if !targets.contains_key(&item.hwnd) && !item.current.approx_eq(item.to, 1) {
                    others.push(AnimItem {
                        hwnd: item.hwnd,
                        from: item.current,
                        to: item.to,
                        current: item.current,
                    });
                }
            }
        }
        (guard.panel_hwnd, cur_map, others)
    };

    let mut items: Vec<AnimItem> = Vec::with_capacity(targets.len() + preserved_others.len());
    items.extend(preserved_others);
    for (&raw, &goal) in targets {
        let hwnd = HWND(raw as *mut _);
        // SAFETY: Validates live window before querying its rect and DWM border delta.
        unsafe {
            if !IsWindow(hwnd).as_bool() || IsIconic(hwnd).as_bool() || IsZoomed(hwnd).as_bool() {
                continue;
            }
            let delta = compute_dwm_border_delta(hwnd);
            let compensated_goal = goal.with_border_delta(delta);
            let from = if let Some(&in_flight) = existing_in_flight.get(&raw) {
                in_flight
            } else {
                let mut cur = RECT::default();
                if GetWindowRect(hwnd, &mut cur).is_err() {
                    compensated_goal
                } else {
                    TileRect::from_win32(cur)
                }
            };
            if from.approx_eq(compensated_goal, 1) {
                continue;
            }
            items.push(AnimItem {
                hwnd: raw,
                from,
                to: compensated_goal,
                current: from,
            });
        }
    }

    if items.is_empty() {
        return;
    }

    if animation_ms < 20 || panel_hwnd == 0 {
        let final_frame: Vec<(isize, TileRect)> = items.iter().map(|it| (it.hwnd, it.to)).collect();
        commit_window_frame(&final_frame);
        if let Ok(mut guard) = state().lock() {
            guard.animation = None;
        }
        return;
    }

    if let Ok(mut guard) = state().lock() {
        guard.animation = Some(ActiveAnimation {
            started_at: Instant::now(),
            duration_ms: animation_ms.clamp(20, 500),
            items,
        });
    }

    // SAFETY: Arms a 10ms (~100Hz) UI timer on `PANEL` to drive smooth non-blocking frames.
    unsafe {
        let _ = SetTimer(HWND(panel_hwnd as *mut _), ID_TILING_ANIM_TIMER, 10, None);
    }
    tick_animation();
}

/// Called from `PANEL`'s `WM_TIMER` (`wParam == ID_TILING_ANIM_TIMER`) to advance active
/// tiling/floating window animations smoothly at ~100Hz without blocking the message loop.
pub(crate) fn tick_animation() {
    let (frame, finished, panel_hwnd) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let panel = guard.panel_hwnd;
        let Some(anim) = guard.animation.as_mut() else {
            return;
        };
        let elapsed_ms = anim.started_at.elapsed().as_secs_f32() * 1000.0;
        let t = (elapsed_ms / anim.duration_ms.max(1) as f32).min(1.0);
        let mut frame = Vec::with_capacity(anim.items.len());
        for item in &mut anim.items {
            let next = if t >= 1.0 {
                item.to
            } else {
                lerp_rect(item.from, item.to, t)
            };
            item.current = next;
            frame.push((item.hwnd, next));
        }
        let done = t >= 1.0;
        if done {
            guard.animation = None;
        }
        (frame, done, panel)
    };

    commit_window_frame(&frame);

    if finished && panel_hwnd != 0 {
        // SAFETY: Cancels the timer on `PANEL` once the animation reaches t = 1.0.
        unsafe {
            let _ = KillTimer(HWND(panel_hwnd as *mut _), ID_TILING_ANIM_TIMER);
        }
    }
}

// ============================================================================
// Public Shell Integration API
// ============================================================================

pub(crate) fn snapshot() -> TilingSnapshot {
    let guard = match state().lock() {
        Ok(g) => g,
        Err(_) => {
            return TilingSnapshot {
                enabled: true,
                direction: TilingDirection::Horizontal,
                inner_gap: 10,
                outer_gap: 12,
                active_hint: true,
                animation_ms: 240,
                tiled_count: 0,
                floating_count: 0,
                focused_is_floating: false,
            };
        }
    };
    let tiled_count = guard.all_tiled_ids().len();
    let focused_is_floating = guard
        .focused_hwnd
        .map(|h| !guard.settings.enabled || guard.floating_windows.contains(&h))
        .unwrap_or(!guard.settings.enabled);
    TilingSnapshot {
        enabled: guard.settings.enabled,
        direction: guard.settings.direction,
        inner_gap: guard.settings.inner_gap,
        outer_gap: guard.settings.outer_gap,
        active_hint: guard.settings.active_hint,
        animation_ms: guard.settings.animation_ms,
        tiled_count,
        floating_count: guard.floating_windows.len(),
        focused_is_floating,
    }
}

pub(crate) fn update_tiling_settings(settings: TilingSettings) {
    if let Ok(mut guard) = state().lock() {
        guard.settings = settings;
    }
    relayout_workspace();
}

pub(crate) fn register_shell_window(hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }
    if let Ok(mut guard) = state().lock() {
        guard.shell_hwnds.insert(hwnd.0 as isize);
    }
}

pub(crate) fn unregister_shell_window(hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }
    if let Ok(mut guard) = state().lock() {
        guard.shell_hwnds.remove(&(hwnd.0 as isize));
    }
}

/// Arms a short one-shot UI timer on `PANEL` after a Windows 11 virtual desktop switch
/// so newly visible desktop windows are discovered and smoothly tiled once the transition settles.
pub(crate) fn schedule_workspace_sync() {
    let panel_hwnd = state().lock().map(|g| g.panel_hwnd).unwrap_or(0);
    if panel_hwnd == 0 {
        return;
    }
    // SAFETY: Arms a 160ms one-shot timer on the owned `PANEL` window.
    unsafe {
        let _ = SetTimer(
            HWND(panel_hwnd as *mut _),
            ID_TILING_WORKSPACE_SYNC_TIMER,
            160,
            None,
        );
    }
}

/// Fired from `PANEL`'s `WM_TIMER` (`wParam == ID_TILING_WORKSPACE_SYNC_TIMER`).
pub(crate) fn handle_workspace_sync_timer() {
    let (panel_hwnd, enabled) = state()
        .lock()
        .map(|g| (g.panel_hwnd, g.settings.enabled))
        .unwrap_or((0, false));
    if panel_hwnd != 0 {
        // SAFETY: Cancels the one-shot workspace sync timer on `PANEL`.
        unsafe {
            let _ = KillTimer(HWND(panel_hwnd as *mut _), ID_TILING_WORKSPACE_SYNC_TIMER);
        }
    }
    if enabled {
        sync_open_windows();
    }
    relayout_workspace();
}

pub(crate) fn is_tiling_enabled() -> bool {
    state()
        .lock()
        .map(|g| g.settings.enabled)
        .unwrap_or(true)
}

pub(crate) fn toggle_tiling_mode() -> bool {
    let (enabled, restore_map, anim_ms) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return true,
        };
        guard.settings.enabled = !guard.settings.enabled;
        let enabled = guard.settings.enabled;
        let restore = if !enabled {
            let mut map = HashMap::new();
            for id in guard.all_tiled_ids() {
                if let Some(&r) = guard.saved_floating_rects.get(&id) {
                    map.insert(id, r);
                }
            }
            map
        } else {
            HashMap::new()
        };
        (enabled, restore, guard.settings.animation_ms)
    };

    if enabled {
        sync_open_windows();
        relayout_workspace();
    } else {
        if !restore_map.is_empty() {
            schedule_animated_placement(&restore_map, anim_ms);
        }
        refresh_active_hint();
    }
    enabled
}

pub(crate) fn set_all_windows_mode(tile_all: bool) {
    let (restore_map, anim_ms) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.settings.enabled = tile_all;
        if tile_all {
            guard.floating_windows.clear();
            (HashMap::new(), guard.settings.animation_ms)
        } else {
            let mut map = HashMap::new();
            for id in guard.all_tiled_ids() {
                if let Some(&r) = guard.saved_floating_rects.get(&id) {
                    map.insert(id, r);
                }
            }
            (map, guard.settings.animation_ms)
        }
    };
    if tile_all {
        sync_open_windows();
        relayout_workspace();
    } else {
        if !restore_map.is_empty() {
            schedule_animated_placement(&restore_map, anim_ms);
        }
        refresh_active_hint();
    }
}

pub(crate) fn toggle_active_window_float() {
    // SAFETY: `GetForegroundWindow` is safe on the UI thread.
    let fg = unsafe { GetForegroundWindow() };
    let target_raw = {
        let guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if !fg.is_invalid() && !guard.shell_hwnds.contains(&(fg.0 as isize)) {
            fg.0 as isize
        } else if let Some(last) = guard.focused_hwnd {
            last
        } else {
            return;
        }
    };
    let target_hwnd = HWND(target_raw as *mut _);
    // SAFETY: Validated non-null HWND.
    let mon_raw = unsafe {
        if !IsWindow(target_hwnd).as_bool() {
            return;
        }
        MonitorFromWindow(target_hwnd, MONITOR_DEFAULTTONEAREST).0 as isize
    };

    let (restore_rect, anim_ms) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if guard.shell_hwnds.contains(&target_raw) {
            return;
        }
        if guard.floating_windows.remove(&target_raw) {
            // Re-tile into this monitor's GlazeWM SplitContainer tree
            guard.settings.enabled = true;
            let dir = guard.settings.direction;
            let focus = guard.focused_hwnd.filter(|&f| f != target_raw);
            let existing_tree = guard.trees_by_monitor.remove(&mon_raw);
            let next_tree = insert_window_into_tree(existing_tree, target_raw, focus, dir);
            guard.trees_by_monitor.insert(mon_raw, next_tree);
            guard.focused_hwnd = Some(target_raw);
            (None, guard.settings.animation_ms)
        } else {
            // Pop out into Floating Mode
            guard.floating_windows.insert(target_raw);
            let _ = guard.remove_from_all_trees(target_raw);
            let work = monitor_work_area(mon_raw, guard.settings.dock_auto_hide, guard.panel_hwnd);
            let float_rect = guard.saved_floating_rects.get(&target_raw).copied().unwrap_or_else(
                || {
                    let w = ((work.width as f32 * 0.64).round() as i32).clamp(480, 1100);
                    let h = ((work.height as f32 * 0.68).round() as i32).clamp(340, 760);
                    TileRect {
                        x: work.x + (work.width - w) / 2,
                        y: work.y + (work.height - h) / 2,
                        width: w,
                        height: h,
                    }
                },
            );
            (Some(float_rect), guard.settings.animation_ms)
        }
    };

    relayout_workspace();
    if let Some(rect) = restore_rect {
        let mut single = HashMap::new();
        single.insert(target_raw, rect);
        schedule_animated_placement(&single, anim_ms);
    }
}

pub(crate) fn flip_split_direction() {
    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.settings.direction = guard.settings.direction.flip();
        let focus = guard.focused_hwnd;
        if let Some(focus_id) = focus {
            if let Some(mon) = guard.find_monitor_of(focus_id) {
                toggle_split_direction_at(guard.trees_by_monitor.get_mut(&mon), Some(focus_id));
            }
        } else if let Some(first_tree) = guard.trees_by_monitor.values_mut().next() {
            toggle_split_direction_at(Some(first_tree), None);
        }
    }
    relayout_workspace();
}

pub(crate) fn set_split_direction(direction: TilingDirection) {
    if let Ok(mut guard) = state().lock() {
        guard.settings.direction = direction;
    }
}

pub(crate) fn swap_focused_window(forward: bool) {
    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(focus_id) = guard.focused_hwnd {
            if let Some(mon) = guard.find_monitor_of(focus_id) {
                swap_window_with_neighbor(guard.trees_by_monitor.get_mut(&mon), focus_id, forward);
            }
        }
    }
    relayout_workspace();
}

pub(crate) fn focus_neighbor_window(forward: bool) {
    let next_hwnd = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let ordered = guard.all_tiled_ids();
        if ordered.len() < 2 {
            return;
        }
        let idx = guard
            .focused_hwnd
            .and_then(|f| ordered.iter().position(|&id| id == f))
            .unwrap_or(0);
        let next_idx = if forward {
            (idx + 1) % ordered.len()
        } else {
            (idx + ordered.len() - 1) % ordered.len()
        };
        let chosen = ordered[next_idx];
        guard.focused_hwnd = Some(chosen);
        chosen
    };

    // SAFETY: Validated live window focus activation on UI thread.
    unsafe {
        let hwnd = HWND(next_hwnd as *mut _);
        if IsWindow(hwnd).as_bool() {
            let _ = SetForegroundWindow(hwnd);
        }
    }
    refresh_active_hint();
}

pub(crate) fn resize_focused_window(delta: f32) {
    let changed = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(focus_id) = guard.focused_hwnd {
            if let Some(mon) = guard.find_monitor_of(focus_id) {
                resize_window_in_tree(guard.trees_by_monitor.get_mut(&mon), focus_id, delta)
            } else {
                false
            }
        } else {
            false
        }
    };
    if changed {
        relayout_workspace();
    }
}

pub(crate) fn adjust_gaps(inner_delta: i32, outer_delta: i32) -> (i32, i32) {
    let res = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return (10, 12),
        };
        guard.settings.inner_gap = (guard.settings.inner_gap + inner_delta).clamp(0, 24);
        guard.settings.outer_gap = (guard.settings.outer_gap + outer_delta).clamp(0, 24);
        (guard.settings.inner_gap, guard.settings.outer_gap)
    };
    relayout_workspace();
    res
}

pub(crate) fn toggle_active_hint() -> bool {
    let enabled = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return true,
        };
        guard.settings.active_hint = !guard.settings.active_hint;
        guard.settings.active_hint
    };
    refresh_active_hint();
    enabled
}

pub(crate) fn cycle_animation_speed() -> u32 {
    let next = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return 240,
        };
        guard.settings.animation_ms = match guard.settings.animation_ms {
            0..=40 => 160,
            41..=200 => 240,
            201..=300 => 340,
            _ => 0,
        };
        guard.settings.animation_ms
    };
    next
}

pub(crate) fn refresh_active_hint() {
    let (ids, focused, active_hint, accent_rgb) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let mut ids = guard.all_tiled_ids();
        for &f in &guard.floating_windows {
            if !ids.contains(&f) {
                ids.push(f);
            }
        }
        for &id in &ids {
            guard.bordered_windows.insert(id);
        }
        (
            ids,
            guard.focused_hwnd,
            guard.settings.active_hint,
            guard.settings.accent_rgb,
        )
    };
    apply_active_hint_borders(&ids, focused, active_hint, accent_rgb);
}

pub(crate) fn relayout_workspace() {
    let vdm = create_virtual_desktop_manager();
    let (enabled, targets, all_ids, focused, active_hint, accent_rgb, anim_ms) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        // 1. Prune closed, hidden, minimized, cloaked, or off-desktop windows cleanly.
        let tracked = guard.all_tiled_ids();
        for id in tracked {
            let hwnd = HWND(id as *mut _);
            // SAFETY: Non-destructive live window status checks.
            let keep = unsafe {
                IsWindow(hwnd).as_bool()
                    && IsWindowVisible(hwnd).as_bool()
                    && !IsIconic(hwnd).as_bool()
                    && !IsZoomed(hwnd).as_bool()
            } && !is_window_cloaked(hwnd)
                && !is_fullscreen_on_monitor(hwnd)
                && is_on_current_virtual_desktop_with(hwnd, vdm.as_ref());
            if !keep {
                guard.remove_from_all_trees(id);
                // SAFETY: Check if window was actually destroyed before discarding saved floating rect.
                if unsafe { !IsWindow(hwnd).as_bool() } {
                    guard.saved_floating_rects.remove(&id);
                    guard.floating_windows.remove(&id);
                    guard.bordered_windows.remove(&id);
                }
            }
        }

        // 2. Compute per-monitor GlazeWM SplitContainer target rectangles.
        let mut combined_targets = HashMap::new();
        if guard.settings.enabled {
            let monitors: Vec<isize> = guard.trees_by_monitor.keys().copied().collect();
            for mon in monitors {
                let work =
                    monitor_work_area(mon, guard.settings.dock_auto_hide, guard.panel_hwnd);
                if let Some(tree) = guard.trees_by_monitor.get(&mon) {
                    let per_mon = compute_tiled_layout(
                        Some(tree),
                        work,
                        guard.settings.inner_gap,
                        guard.settings.outer_gap,
                    );
                    for (id, rect) in per_mon {
                        guard.last_tiled_rects.insert(id, rect);
                        guard.bordered_windows.insert(id);
                        combined_targets.insert(id, rect);
                    }
                }
            }
        }

        let ids = guard.all_tiled_ids();
        (
            guard.settings.enabled,
            combined_targets,
            ids,
            guard.focused_hwnd,
            guard.settings.active_hint,
            guard.settings.accent_rgb,
            guard.settings.animation_ms,
        )
    };

    if enabled && !targets.is_empty() {
        schedule_animated_placement(&targets, anim_ms);
    }
    apply_active_hint_borders(&all_ids, focused, active_hint, accent_rgb);
}

unsafe extern "system" fn enum_windows_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
    list.push(hwnd);
    BOOL(1)
}

pub(crate) fn sync_open_windows() {
    let mut found: Vec<HWND> = Vec::new();
    // SAFETY: `found` lives on the stack for the synchronous duration of `EnumWindows`.
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_cb),
            LPARAM(&mut found as *mut Vec<HWND> as isize),
        );
    }

    let vdm = create_virtual_desktop_manager();
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let shell_hwnds = guard.shell_hwnds.clone();
    for hwnd in found {
        if !should_manage_window(hwnd, &shell_hwnds, vdm.as_ref()) {
            continue;
        }
        let raw = hwnd.0 as isize;
        if guard.floating_windows.contains(&raw) || guard.find_monitor_of(raw).is_some() {
            continue;
        }
        // SAFETY: Validated live window handle.
        let mon_raw = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST).0 as isize };
        if let Some(vis_rect) = window_visual_rect(hwnd) {
            guard.saved_floating_rects.entry(raw).or_insert(vis_rect);
        }
        let dir = guard.settings.direction;
        let focus = guard.focused_hwnd;
        let existing_tree = guard.trees_by_monitor.remove(&mon_raw);
        let next_tree = insert_window_into_tree(existing_tree, raw, focus, dir);
        guard.trees_by_monitor.insert(mon_raw, next_tree);
    }
}

/// Win32 `SetWinEventHook` callback — delegates immediately to safe Rust handlers.
unsafe extern "system" fn win_event_callback(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if id_object != OBJID_WINDOW || id_child != 0 || hwnd.is_invalid() {
        return;
    }
    handle_win_event(event, hwnd);
}

fn handle_win_event(event: u32, hwnd: HWND) {
    let raw = hwnd.0 as isize;
    match event {
        EVENT_SYSTEM_FOREGROUND => {
            let vdm = create_virtual_desktop_manager();
            let (ids, active_hint, accent_rgb, needs_relayout) = {
                let mut guard = match state().lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                if guard.shell_hwnds.contains(&raw) {
                    return;
                }
                guard.focused_hwnd = Some(raw);
                guard.bordered_windows.insert(raw);

                let mut needs_relayout = false;
                if guard.settings.enabled && !guard.floating_windows.contains(&raw) {
                    let in_tree = guard.find_monitor_of(raw).is_some();
                    let eligible = should_manage_window(hwnd, &guard.shell_hwnds, vdm.as_ref());
                    if in_tree && !eligible {
                        // Window became maximized or fullscreen -> smoothly expand remaining tiles.
                        guard.remove_from_all_trees(raw);
                        needs_relayout = true;
                    } else if !in_tree && eligible {
                        // Window restored from maximized/minimized -> smoothly re-tile.
                        // SAFETY: Validated live HWND.
                        let mon_raw =
                            unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST).0 as isize };
                        if let Some(vis_rect) = window_visual_rect(hwnd) {
                            guard.saved_floating_rects.entry(raw).or_insert(vis_rect);
                        }
                        let dir = guard.settings.direction;
                        let existing = guard.trees_by_monitor.remove(&mon_raw);
                        let next_tree = insert_window_into_tree(existing, raw, None, dir);
                        guard.trees_by_monitor.insert(mon_raw, next_tree);
                        needs_relayout = true;
                    }
                }

                (
                    guard.all_tiled_ids(),
                    guard.settings.active_hint,
                    guard.settings.accent_rgb,
                    needs_relayout,
                )
            };
            if needs_relayout {
                relayout_workspace();
            } else {
                apply_active_hint_borders(&ids, Some(raw), active_hint, accent_rgb);
            }
        }
        EVENT_OBJECT_SHOW | EVENT_SYSTEM_MINIMIZEEND => {
            let vdm = create_virtual_desktop_manager();
            let should_relayout = {
                let mut guard = match state().lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                if !guard.settings.enabled || guard.floating_windows.contains(&raw) {
                    return;
                }
                if !should_manage_window(hwnd, &guard.shell_hwnds, vdm.as_ref())
                    || guard.find_monitor_of(raw).is_some()
                {
                    return;
                }
                // SAFETY: Validated live HWND.
                let mon_raw =
                    unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST).0 as isize };
                if let Some(vis_rect) = window_visual_rect(hwnd) {
                    guard.saved_floating_rects.entry(raw).or_insert(vis_rect);
                }
                let dir = guard.settings.direction;
                let focus = guard.focused_hwnd;
                let existing = guard.trees_by_monitor.remove(&mon_raw);
                let next_tree = insert_window_into_tree(existing, raw, focus, dir);
                guard.trees_by_monitor.insert(mon_raw, next_tree);
                guard.focused_hwnd = Some(raw);
                true
            };
            if should_relayout {
                relayout_workspace();
            }
        }
        EVENT_SYSTEM_MINIMIZESTART | EVENT_OBJECT_HIDE | EVENT_OBJECT_DESTROY => {
            let removed = {
                let mut guard = match state().lock() {
                    Ok(g) => g,
                    Err(_) => return,
                };
                let did_remove = guard.remove_from_all_trees(raw);
                if event == EVENT_OBJECT_DESTROY {
                    guard.saved_floating_rects.remove(&raw);
                    guard.floating_windows.remove(&raw);
                    guard.bordered_windows.remove(&raw);
                }
                if guard.focused_hwnd == Some(raw) {
                    guard.focused_hwnd = guard.all_tiled_ids().last().copied();
                }
                did_remove
            };
            if removed {
                relayout_workspace();
            }
        }
        EVENT_SYSTEM_MOVESIZEEND => {
            handle_move_size_end(hwnd, raw);
        }
        _ => {}
    }
}

/// GlazeWM / COSMIC interactive mouse drag-to-swap, cross-monitor drop, and edge-resize handler.
fn handle_move_size_end(hwnd: HWND, raw: isize) {
    // SAFETY: Validated live HWND from WinEvent callback.
    let new_mon_raw = unsafe {
        if !IsWindow(hwnd).as_bool() {
            return;
        }
        MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST).0 as isize
    };
    let Some(actual_rect) = window_visual_rect(hwnd) else {
        return;
    };

    let should_relayout = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        // Ignore move/size notifications caused by our own active animation frames.
        if guard.animation.is_some() {
            return;
        }
        if !guard.settings.enabled || guard.floating_windows.contains(&raw) {
            // Save updated visual floating geometry so future Tile -> Float restores user's placement.
            guard.saved_floating_rects.insert(raw, actual_rect);
            return;
        }
        let Some(old_mon_raw) = guard.find_monitor_of(raw) else {
            return;
        };

        // Case 1: Dragged onto another monitor -> migrate to destination monitor's tree.
        if old_mon_raw != new_mon_raw {
            guard.remove_from_all_trees(raw);
            let dir = guard.settings.direction;
            let dest_tree = guard.trees_by_monitor.remove(&new_mon_raw);
            let updated = insert_window_into_tree(dest_tree, raw, None, dir);
            guard.trees_by_monitor.insert(new_mon_raw, updated);
            true
        } else {
            // Case 2: Dropped onto another tiled sibling on the same monitor -> swap tiles!
            let (cx, cy) = actual_rect.center();
            let drop_target = guard
                .last_tiled_rects
                .iter()
                .find_map(|(&other_id, &other_rect)| {
                    if other_id != raw && other_rect.contains_point(cx, cy) {
                        Some(other_id)
                    } else {
                        None
                    }
                });

            if let Some(other_id) = drop_target {
                if let Some(tree) = guard.trees_by_monitor.get_mut(&old_mon_raw) {
                    swap_window_ids(tree, raw, other_id);
                }
                true
            } else if let Some(&prev_rect) = guard.last_tiled_rects.get(&raw) {
                // Case 3: Resized a tiled edge -> adjust parent SplitContainer proportional weight!
                let dw = actual_rect.width - prev_rect.width;
                let dh = actual_rect.height - prev_rect.height;
                if let Some(tree) = guard.trees_by_monitor.get_mut(&old_mon_raw) {
                    if let Some(pdir) = parent_direction_of(tree, raw) {
                        let ratio_delta = match pdir {
                            TilingDirection::Horizontal if dw.abs() > 12 => {
                                dw as f32 / prev_rect.width.max(120) as f32
                            }
                            TilingDirection::Vertical if dh.abs() > 12 => {
                                dh as f32 / prev_rect.height.max(120) as f32
                            }
                            _ => 0.0,
                        };
                        if ratio_delta.abs() > 0.03 {
                            resize_window_in_tree(Some(tree), raw, ratio_delta.clamp(-0.6, 0.6));
                        }
                    }
                }
                true
            } else {
                true
            }
        }
    };

    if should_relayout {
        relayout_workspace();
    }
}

pub(crate) fn init_tiling_hooks(
    panel_hwnd: HWND,
    shell_windows: &[HWND],
    settings: TilingSettings,
) {
    {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.panel_hwnd = panel_hwnd.0 as isize;
        guard.settings = settings;
        for &hwnd in shell_windows {
            if !hwnd.is_invalid() {
                guard.shell_hwnds.insert(hwnd.0 as isize);
            }
        }
        let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
        let ranges = [
            (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
            (EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND),
            (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
            (EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE),
        ];
        // SAFETY: Registers out-of-context WinEvent hooks on the UI thread;
        // every handle is stored and cleanly released in `cleanup_tiling_hooks`.
        unsafe {
            for (min_ev, max_ev) in ranges {
                let hook = SetWinEventHook(
                    min_ev,
                    max_ev,
                    None,
                    Some(win_event_callback),
                    0,
                    0,
                    flags,
                );
                if !hook.is_invalid() {
                    guard.event_hooks.push(hook.0 as isize);
                }
            }
        }
    }
    if settings.enabled {
        sync_open_windows();
        relayout_workspace();
    }
}

pub(crate) fn cleanup_tiling_hooks() {
    let (hooks, bordered, panel_hwnd) = {
        let mut guard = match state().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        guard.animation = None;
        let hooks = std::mem::take(&mut guard.event_hooks);
        let bordered: Vec<isize> = guard.bordered_windows.drain().collect();
        (hooks, bordered, guard.panel_hwnd)
    };
    // SAFETY: Releases timers, unhooks WinEvent hooks, and restores default Windows 11 borders.
    unsafe {
        if panel_hwnd != 0 {
            let panel = HWND(panel_hwnd as *mut _);
            let _ = KillTimer(panel, ID_TILING_ANIM_TIMER);
            let _ = KillTimer(panel, ID_TILING_WORKSPACE_SYNC_TIMER);
        }
        for raw in hooks {
            let _ = UnhookWinEvent(HWINEVENTHOOK(raw as *mut _));
        }
        for raw in bordered {
            let hwnd = HWND(raw as *mut _);
            if IsWindow(hwnd).as_bool() {
                let default_col = DWMWA_COLOR_DEFAULT;
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_BORDER_COLOR,
                    &default_col as *const u32 as *const _,
                    size_of::<u32>() as u32,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_container_inserts_computes_gaps_and_flattens_on_remove() {
        let work = TileRect {
            x: 0,
            y: 38,
            width: 1440,
            height: 780,
        };
        let mut tree = Some(insert_window_into_tree(
            None,
            101,
            None,
            TilingDirection::Horizontal,
        ));
        tree = Some(insert_window_into_tree(
            tree,
            102,
            Some(101),
            TilingDirection::Horizontal,
        ));
        tree = Some(insert_window_into_tree(
            tree,
            103,
            Some(102),
            TilingDirection::Vertical,
        ));

        let layout = compute_tiled_layout(tree.as_ref(), work, 10, 12);
        assert_eq!(layout.len(), 3);
        let r1 = layout[&101];
        let r2 = layout[&102];
        let r3 = layout[&103];
        assert_eq!(r1.x, 12);
        assert_eq!(r2.x - (r1.x + r1.width), 10);
        assert_eq!(r3.y - (r2.y + r2.height), 10);

        let flattened = remove_window_from_tree(tree, 103);
        assert_eq!(collect_window_ids(flattened.as_ref()), vec![101, 102]);
    }

    #[test]
    fn resize_and_swap_preserve_bounds_and_order() {
        let work = TileRect {
            x: 0,
            y: 38,
            width: 1440,
            height: 780,
        };
        let mut tree = Some(insert_window_into_tree(
            None,
            201,
            None,
            TilingDirection::Horizontal,
        ));
        tree = Some(insert_window_into_tree(
            tree,
            202,
            Some(201),
            TilingDirection::Horizontal,
        ));
        let before = compute_tiled_layout(tree.as_ref(), work, 10, 12);
        assert!(resize_window_in_tree(tree.as_mut(), 201, 0.35));
        let after = compute_tiled_layout(tree.as_ref(), work, 10, 12);
        assert!(after[&201].width > before[&201].width);

        swap_window_with_neighbor(tree.as_mut(), 201, true);
        assert_eq!(collect_window_ids(tree.as_ref()), vec![202, 201]);
    }

    #[test]
    fn cubic_out_lerp_and_border_delta_compensation_work() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        let a = TileRect {
            x: 0,
            y: 0,
            width: 400,
            height: 400,
        };
        let b = TileRect {
            x: 200,
            y: 100,
            width: 800,
            height: 600,
        };
        let mid = lerp_rect(a, b, 0.5);
        assert!(mid.x > 100 && mid.x < 200);

        let delta = RectDelta {
            left: 7,
            top: 0,
            right: 7,
            bottom: 7,
        };
        let comp = b.with_border_delta(delta);
        assert_eq!(comp.x, 193);
        assert_eq!(comp.width, 814);
        assert_eq!(comp.height, 607);
    }

    #[test]
    fn split_flip_and_drop_target_geometry_work() {
        let mut tree = Some(insert_window_into_tree(
            None,
            301,
            None,
            TilingDirection::Horizontal,
        ));
        tree = Some(insert_window_into_tree(
            tree,
            302,
            Some(301),
            TilingDirection::Horizontal,
        ));
        assert_eq!(
            parent_direction_of(tree.as_ref().unwrap(), 301),
            Some(TilingDirection::Horizontal)
        );
        toggle_split_direction_at(tree.as_mut(), Some(301));
        assert_eq!(
            parent_direction_of(tree.as_ref().unwrap(), 301),
            Some(TilingDirection::Vertical)
        );

        let slot = TileRect {
            x: 100,
            y: 50,
            width: 500,
            height: 400,
        };
        let (cx, cy) = slot.center();
        assert_eq!((cx, cy), (350, 250));
        assert!(slot.contains_point(cx, cy));
        assert!(!slot.contains_point(50, 250));
    }
}
