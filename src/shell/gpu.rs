//! GPU-style SDF Surface Compositor & High-DPI Icon Pipeline for Luma Shell.
//!
//! Provides sub-pixel Signed Distance Field (SDF) rounded rectangle, full-pill
//! stadium capsule, and circular halo rasterization, COSMIC Desktop top-edge
//! specular rim highlights, smooth cosine magnification interpolation, and
//! high-quality area-averaged 32-bit ARGB icon downsampling for 256x256
//! Windows Shell (`SHIL_JUMBO`) icons.

use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDIBits,
    GetObjectW, HDC,
};
use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, HICON, ICONINFO};

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

/// Renders a COSMIC-styled floating dock pill or panel surface into a 32-bit premultiplied
/// ARGB buffer using a sub-pixel Signed Distance Field (SDF) with vertical glass
/// luminance gradient and a crisp top-edge specular rim highlight.
///
/// # Safety
/// `bits` must point to a valid 32-bit ARGB buffer of at least `(w * h)` `u32` elements.
pub(crate) unsafe fn render_dock_background_argb(
    bits: *mut u32,
    w: i32,
    h: i32,
    radius: f32,
    bg_color: u32,
    border_color: u32,
    base_alpha: u8,
) {
    if bits.is_null() || w <= 0 || h <= 0 {
        return;
    }
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let inner_w = (half_w - radius).max(0.0);
    let inner_h = (half_h - radius).max(0.0);

    let bg_r = ((bg_color >> 16) & 0xFF) as f32;
    let bg_g = ((bg_color >> 8) & 0xFF) as f32;
    let bg_b = (bg_color & 0xFF) as f32;

    let br_r = ((border_color >> 16) & 0xFF) as f32;
    let br_g = ((border_color >> 8) & 0xFF) as f32;
    let br_b = (border_color & 0xFF) as f32;

    let inv_h = 1.0 / (h.max(1) as f32);

    for y in 0..h {
        let py = y as f32 + 0.5;
        let dy = (py - half_h).abs() - inner_h;
        let row_offset = (y * w) as usize;
        // Subtle vertical COSMIC glass sheen (brighter at top, deeper at base)
        let v_grad = 1.05 - (py * inv_h) * 0.10;
        let top_specular = if y <= 2 { 0.24 * (1.0 - y as f32 * 0.4) } else { 0.0 };

        for x in 0..w {
            let px = x as f32 + 0.5;
            let dx = (px - half_w).abs() - inner_w;

            let dist = if dx > 0.0 && dy > 0.0 {
                (dx * dx + dy * dy).sqrt() - radius
            } else if dx > 0.0 {
                dx - radius
            } else if dy > 0.0 {
                dy - radius
            } else {
                dx.max(dy) - radius
            };

            let pixel_ptr = unsafe { bits.add(row_offset + x as usize) };

            if dist > 1.0 {
                unsafe { *pixel_ptr = 0 };
            } else if dist < -1.5 {
                let a = base_alpha as f32 / 255.0;
                let r = (bg_r * v_grad).min(255.0);
                let g = (bg_g * v_grad).min(255.0);
                let b = (bg_b * v_grad).min(255.0);
                let pr = (r * a).round() as u32;
                let pg = (g * a).round() as u32;
                let pb = (b * a).round() as u32;
                unsafe {
                    *pixel_ptr = ((base_alpha as u32) << 24) | (pr << 16) | (pg << 8) | pb;
                }
            } else {
                let coverage = ((1.0 - dist) * 0.5).clamp(0.0, 1.0);
                let edge_alpha = (coverage * base_alpha as f32).round() as u32;

                let border_mix =
                    ((1.0 - (dist + 0.5).abs().min(1.0)).max(0.0) + top_specular).min(1.0);
                let r = (bg_r * v_grad) * (1.0 - border_mix) + br_r * border_mix;
                let g = (bg_g * v_grad) * (1.0 - border_mix) + br_g * border_mix;
                let b = (bg_b * v_grad) * (1.0 - border_mix) + br_b * border_mix;

                let a = edge_alpha as f32 / 255.0;
                let pr = (r.min(255.0) * a).round() as u32;
                let pg = (g.min(255.0) * a).round() as u32;
                let pb = (b.min(255.0) * a).round() as u32;
                unsafe {
                    *pixel_ptr = (edge_alpha << 24) | (pr << 16) | (pg << 8) | pb;
                }
            }
        }
    }
}

/// Alpha-blends a sub-pixel anti-aliased circle (used for COSMIC dock active/hover halos
/// and crisp circular running indicator dots) into a 32-bit premultiplied ARGB buffer.
///
/// # Safety
/// `bits` must point to a valid 32-bit ARGB buffer of at least `(w * h)` `u32` elements.
pub(crate) unsafe fn fill_circle_argb(
    bits: *mut u32,
    w: i32,
    h: i32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: u32,
    alpha_u8: u8,
) {
    if bits.is_null() || w <= 0 || h <= 0 || radius <= 0.0 || alpha_u8 == 0 {
        return;
    }
    let x0 = ((cx - radius - 1.5).floor() as i32).max(0);
    let y0 = ((cy - radius - 1.5).floor() as i32).max(0);
    let x1 = ((cx + radius + 1.5).ceil() as i32).min(w);
    let y1 = ((cy + radius + 1.5).ceil() as i32).min(h);

    let sr = ((color >> 16) & 0xFF) as f32;
    let sg = ((color >> 8) & 0xFF) as f32;
    let sb = (color & 0xFF) as f32;
    let a_norm = alpha_u8 as f32 / 255.0;

    for y in y0..y1 {
        let py = y as f32 + 0.5 - cy;
        let row_offset = (y * w) as usize;
        for x in x0..x1 {
            let px = x as f32 + 0.5 - cx;
            let dist = (px * px + py * py).sqrt() - radius;
            if dist <= 1.0 {
                let cov = ((1.0 - dist) * 0.5).clamp(0.0, 1.0);
                let alpha = (cov * a_norm * 255.0).round() as u32;
                if alpha == 0 {
                    continue;
                }
                let pr = (sr * (alpha as f32 / 255.0)).round() as u32;
                let pg = (sg * (alpha as f32 / 255.0)).round() as u32;
                let pb = (sb * (alpha as f32 / 255.0)).round() as u32;

                let p = unsafe { bits.add(row_offset + x as usize) };
                let inv_a = 255 - alpha;
                let cur = unsafe { *p };
                let cur_a = (cur >> 24) & 0xFF;
                let cur_r = (cur >> 16) & 0xFF;
                let cur_g = (cur >> 8) & 0xFF;
                let cur_b = cur & 0xFF;

                let out_a = alpha + (cur_a * inv_a) / 255;
                let out_r = pr + (cur_r * inv_a) / 255;
                let out_g = pg + (cur_g * inv_a) / 255;
                let out_b = pb + (cur_b * inv_a) / 255;
                unsafe {
                    *p = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }
}

/// Alpha-blends a sub-pixel anti-aliased rounded rectangle into an existing 32-bit
/// premultiplied ARGB surface buffer.
///
/// # Safety
/// `bits` must point to a valid 32-bit ARGB buffer of at least `(w * h)` `u32` elements.
pub(crate) unsafe fn fill_rounded_box_argb(
    bits: *mut u32,
    w: i32,
    h: i32,
    rect: &RECT,
    radius: f32,
    box_color: u32,
    box_alpha: u8,
) {
    if bits.is_null() || w <= 0 || h <= 0 {
        return;
    }
    let bw = rect.right - rect.left;
    let bh = rect.bottom - rect.top;
    if bw <= 0 || bh <= 0 {
        return;
    }
    let cx = rect.left as f32 + bw as f32 / 2.0;
    let cy = rect.top as f32 + bh as f32 / 2.0;
    let inner_w = (bw as f32 / 2.0 - radius).max(0.0);
    let inner_h = (bh as f32 / 2.0 - radius).max(0.0);

    let br = ((box_color >> 16) & 0xFF) as f32;
    let bg = ((box_color >> 8) & 0xFF) as f32;
    let bb = (box_color & 0xFF) as f32;
    let a_norm = box_alpha as f32 / 255.0;

    for y in rect.top.max(0)..rect.bottom.min(h) {
        let py = y as f32 + 0.5;
        let dy = (py - cy).abs() - inner_h;
        let row_offset = (y * w) as usize;

        for x in rect.left.max(0)..rect.right.min(w) {
            let px = x as f32 + 0.5;
            let dx = (px - cx).abs() - inner_w;

            let dist = if dx > 0.0 && dy > 0.0 {
                (dx * dx + dy * dy).sqrt() - radius
            } else if dx > 0.0 {
                dx - radius
            } else if dy > 0.0 {
                dy - radius
            } else {
                dx.max(dy) - radius
            };

            if dist <= 1.0 {
                let cov = ((1.0 - dist) * 0.5).clamp(0.0, 1.0);
                let alpha = (cov * a_norm * 255.0).round() as u32;
                let pr = (br * (alpha as f32 / 255.0)).round() as u32;
                let pg = (bg * (alpha as f32 / 255.0)).round() as u32;
                let pb = (bb * (alpha as f32 / 255.0)).round() as u32;
                let p = unsafe { bits.add(row_offset + x as usize) };

                let inv_a = 255 - alpha;
                let cur = unsafe { *p };
                let cur_a = (cur >> 24) & 0xFF;
                let cur_r = (cur >> 16) & 0xFF;
                let cur_g = (cur >> 8) & 0xFF;
                let cur_b = cur & 0xFF;

                let out_a = alpha + (cur_a * inv_a) / 255;
                let out_r = pr + (cur_r * inv_a) / 255;
                let out_g = pg + (cur_g * inv_a) / 255;
                let out_b = pb + (cur_b * inv_a) / 255;

                unsafe {
                    *p = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                }
            }
        }
    }
}

/// Composites an `HICON` (including 256x256 `SHIL_JUMBO` icons) into a 32-bit
/// premultiplied ARGB buffer using smooth area-box downsampling / bilinear
/// filtering instead of nearest-neighbor GDI `DrawIconEx`.
///
/// Returns `true` if the icon was extracted and composited smoothly.
///
/// # Safety
/// `bits` must point to a valid 32-bit ARGB buffer of at least `(w * h)` `u32` elements,
/// and `hdc` must be a valid compatible device context.
pub(crate) unsafe fn draw_icon_high_quality_argb(
    bits: *mut u32,
    w: i32,
    h: i32,
    hdc: HDC,
    icon: HICON,
    dst_x: i32,
    dst_y: i32,
    dst_size: i32,
) -> bool {
    if bits.is_null() || w <= 0 || h <= 0 || icon.is_invalid() || dst_size <= 0 {
        return false;
    }

    let mut info = ICONINFO::default();
    if unsafe { GetIconInfo(icon, &mut info) }.is_err() {
        return false;
    }

    let mut success = false;

    if !info.hbmColor.is_invalid() {
        let mut bmp = BITMAP::default();
        let got = unsafe {
            GetObjectW(
                info.hbmColor,
                size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut c_void),
            )
        };
        let src_w = bmp.bmWidth;
        let src_h = bmp.bmHeight.abs();
        if got > 0 && src_w > 0 && src_h > 0 && src_w <= 1024 && src_h <= 1024 {
            let mut bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: src_w,
                    biHeight: -src_h, // top-down scanlines
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let pixel_count = (src_w as usize) * (src_h as usize);
            let mut src_pixels = vec![0u32; pixel_count];
            let lines = unsafe {
                GetDIBits(
                    hdc,
                    info.hbmColor,
                    0,
                    src_h as u32,
                    Some(src_pixels.as_mut_ptr() as *mut c_void),
                    &mut bi,
                    DIB_RGB_COLORS,
                )
            };

            if lines > 0 {
                // Detect if the icon bitmap contains real alpha channel values; if all 0, synthesize from hbmMask
                let has_alpha = src_pixels.iter().any(|&p| (p >> 24) != 0);
                if !has_alpha && !info.hbmMask.is_invalid() {
                    let mut mask_pixels = vec![0u32; pixel_count];
                    let mask_lines = unsafe {
                        GetDIBits(
                            hdc,
                            info.hbmMask,
                            0,
                            src_h as u32,
                            Some(mask_pixels.as_mut_ptr() as *mut c_void),
                            &mut bi,
                            DIB_RGB_COLORS,
                        )
                    };
                    if mask_lines > 0 {
                        for i in 0..pixel_count {
                            // Mask white (0xFFFFFF) = transparent, black (0x000000) = opaque
                            let is_transparent = (mask_pixels[i] & 0x00FFFFFF) != 0;
                            if !is_transparent {
                                src_pixels[i] |= 0xFF000000;
                            } else {
                                src_pixels[i] = 0;
                            }
                        }
                    }
                }

                // Trim empty transparent padding on SHIL_JUMBO 256x256 icons that only fill top-left 48x48
                let mut min_x = src_w;
                let mut min_y = src_h;
                let mut max_x = -1i32;
                let mut max_y = -1i32;
                for sy in 0..src_h {
                    let row = (sy * src_w) as usize;
                    for sx in 0..src_w {
                        if (src_pixels[row + sx as usize] >> 24) > 8 {
                            if sx < min_x {
                                min_x = sx;
                            }
                            if sx > max_x {
                                max_x = sx;
                            }
                            if sy < min_y {
                                min_y = sy;
                            }
                            if sy > max_y {
                                max_y = sy;
                            }
                        }
                    }
                }

                let (crop_x, crop_y, crop_w, crop_h) = if max_x >= min_x && max_y >= min_y {
                    let bw = max_x - min_x + 1;
                    let bh = max_y - min_y + 1;
                    // If the shell packed a small icon (e.g. 48x48) into the corner of a 256x256 jumbo sheet,
                    // crop to its square bounding box so it renders full-size instead of tiny.
                    if src_w >= 128 && bw < src_w / 2 && bh < src_h / 2 {
                        let side = bw.max(bh);
                        let cx = (min_x + bw / 2 - side / 2).max(0).min(src_w - side);
                        let cy = (min_y + bh / 2 - side / 2).max(0).min(src_h - side);
                        (cx, cy, side, side)
                    } else {
                        (0, 0, src_w, src_h)
                    }
                } else {
                    (0, 0, src_w, src_h)
                };

                let scale_x = crop_w as f32 / dst_size as f32;
                let scale_y = crop_h as f32 / dst_size as f32;

                for dy in 0..dst_size {
                    let out_y = dst_y + dy;
                    if out_y < 0 || out_y >= h {
                        continue;
                    }
                    let sy0 = crop_y as f32 + dy as f32 * scale_y;
                    let sy1 = (sy0 + scale_y).min((crop_y + crop_h) as f32);
                    let iy0 = (sy0.floor() as i32).clamp(0, src_h - 1);
                    let iy1 = (sy1.ceil() as i32).clamp(iy0 + 1, src_h);

                    let row_offset = (out_y * w) as usize;
                    for dx in 0..dst_size {
                        let out_x = dst_x + dx;
                        if out_x < 0 || out_x >= w {
                            continue;
                        }
                        let sx0 = crop_x as f32 + dx as f32 * scale_x;
                        let sx1 = (sx0 + scale_x).min((crop_x + crop_w) as f32);
                        let ix0 = (sx0.floor() as i32).clamp(0, src_w - 1);
                        let ix1 = (sx1.ceil() as i32).clamp(ix0 + 1, src_w);

                        // Area-averaged box filter in premultiplied alpha space
                        let mut acc_a = 0.0f32;
                        let mut acc_r = 0.0f32;
                        let mut acc_g = 0.0f32;
                        let mut acc_b = 0.0f32;
                        let mut weight_sum = 0.0f32;

                        for sy in iy0..iy1 {
                            let wy = ((sy + 1) as f32).min(sy1) - (sy as f32).max(sy0);
                            if wy <= 0.0 {
                                continue;
                            }
                            let srow = (sy * src_w) as usize;
                            for sx in ix0..ix1 {
                                let wx = ((sx + 1) as f32).min(sx1) - (sx as f32).max(sx0);
                                let wt = wx * wy;
                                if wt <= 0.0 {
                                    continue;
                                }
                                let px = src_pixels[srow + sx as usize];
                                let a = ((px >> 24) & 0xFF) as f32 / 255.0;
                                let r = ((px >> 16) & 0xFF) as f32;
                                let g = ((px >> 8) & 0xFF) as f32;
                                let b = (px & 0xFF) as f32;
                                acc_a += a * wt;
                                acc_r += r * a * wt;
                                acc_g += g * a * wt;
                                acc_b += b * a * wt;
                                weight_sum += wt;
                            }
                        }

                        if weight_sum > 0.0 && acc_a > 0.002 {
                            let norm_a = (acc_a / weight_sum).clamp(0.0, 1.0);
                            let pr = (acc_r / weight_sum).clamp(0.0, 255.0) as u32;
                            let pg = (acc_g / weight_sum).clamp(0.0, 255.0) as u32;
                            let pb = (acc_b / weight_sum).clamp(0.0, 255.0) as u32;
                            let pa = (norm_a * 255.0).round() as u32;

                            let inv_a = 255 - pa;
                            let p = unsafe { bits.add(row_offset + out_x as usize) };
                            let cur = unsafe { *p };
                            let cur_a = (cur >> 24) & 0xFF;
                            let cur_r = (cur >> 16) & 0xFF;
                            let cur_g = (cur >> 8) & 0xFF;
                            let cur_b = cur & 0xFF;

                            let out_a = (pa + (cur_a * inv_a) / 255).min(255);
                            let out_r = (pr + (cur_r * inv_a) / 255).min(255);
                            let out_g = (pg + (cur_g * inv_a) / 255).min(255);
                            let out_b = (pb + (cur_b * inv_a) / 255).min(255);

                            unsafe {
                                *p = (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                            }
                        }
                    }
                }
                success = true;
            }
        }
    }

    if !info.hbmColor.is_invalid() {
        let _ = unsafe { DeleteObject(info.hbmColor) };
    }
    if !info.hbmMask.is_invalid() {
        let _ = unsafe { DeleteObject(info.hbmMask) };
    }
    success
}

/// Ensures GDI-drawn icons within an ARGB DIB section retain opaque alpha values.
///
/// # Safety
/// `bits` must point to a valid 32-bit ARGB buffer of at least `(w * h)` `u32` elements.
pub(crate) unsafe fn ensure_dib_alpha(
    bits: *mut u32,
    w: i32,
    h: i32,
    left: i32,
    top: i32,
    icon_w: i32,
    icon_h: i32,
) {
    if bits.is_null() || w <= 0 || h <= 0 {
        return;
    }
    let x0 = left.max(0);
    let y0 = top.max(0);
    let x1 = (left + icon_w).min(w);
    let y1 = (top + icon_h).min(h);

    for y in y0..y1 {
        let row_offset = (y * w) as usize;
        for x in x0..x1 {
            let p = unsafe { bits.add(row_offset + x as usize) };
            let val = unsafe { *p };
            let a = (val >> 24) & 0xFF;
            let rgb = val & 0x00FFFFFF;
            if a == 0 && rgb != 0 {
                unsafe {
                    *p = 0xFF000000 | rgb;
                }
            }
        }
    }
}
