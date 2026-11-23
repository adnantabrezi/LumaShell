export const RUST_GPU_RS_SOURCE = `//! src/shell/gpu.rs
//! GPU-style SDF Surface Compositor & Shader Pipeline for Luma Shell.

use windows::Win32::Foundation::RECT;

/// Smooth cosine magnification curve matching the WGPU/WebGL2 dock compositor.
pub(crate) fn cosmic_magnification_factor(
    pointer_x: i32,
    slot_cx: i32,
    envelope: f32,
    magnification_pct: u8,
) -> f32 {
    if pointer_x < 0 || magnification_pct == 0 {
        return 0.0;
    }
    let distance = (pointer_x - slot_cx).abs() as f32;
    let radius = 115.0;
    if distance >= radius {
        return 0.0;
    }
    let norm = 1.0 - (distance / radius);
    let smooth = 0.5 * (1.0 - (norm * std::f32::consts::PI).cos());
    smooth * envelope * (magnification_pct as f32 / 45.0)
}

pub(crate) unsafe fn render_dock_background_argb(
    bits: *mut u32,
    w: i32,
    h: i32,
    radius: f32,
    bg_color: u32,
    border_color: u32,
    base_alpha: u8,
) {
    // Sub-pixel Signed Distance Field (SDF) rasterizer with vertical
    // COSMIC glass luminance gradient & top-edge specular rim highlight
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let inner_w = (half_w - radius).max(0.0);
    let inner_h = (half_h - radius).max(0.0);
    // ... full premultiplied ARGB composition written to src/shell/gpu.rs
}`;

export const RUST_ICED_WGPU_SOURCE = `//! src/bin/luma_iced.rs
//! Standalone GPU-accelerated binary: cargo run --bin luma_iced
//! Powered by iced 0.13 + wgpu with full COSMIC Dock & 3-Zone Applet Settings.

use iced::widget::{button, checkbox, column, container, row, scrollable, slider, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme};

// Full implementation available in /src/bin/luma_iced.rs
// Syncs directly with %APPDATA%\\LumaShell\\settings.ini`;
