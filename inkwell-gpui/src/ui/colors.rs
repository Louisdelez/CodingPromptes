#![allow(dead_code)]
use gpui::*;
use std::cell::RefCell;

thread_local! {
    static DARK_MODE: RefCell<bool> = const { RefCell::new(true) };
}

pub fn set_dark_mode(dark: bool) { DARK_MODE.with(|d| *d.borrow_mut() = dark); }
pub fn is_dark() -> bool { DARK_MODE.with(|d| *d.borrow()) }

pub fn bg_primary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.15, 0.07, 1.0) } else { hsla(0.0, 0.0, 1.0, 1.0) } }
pub fn bg_secondary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.12, 0.10, 1.0) } else { hsla(220.0/360.0, 0.10, 0.97, 1.0) } }
pub fn bg_tertiary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.10, 0.14, 1.0) } else { hsla(220.0/360.0, 0.08, 0.93, 1.0) } }
pub fn border_c() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.10, 0.20, 1.0) } else { hsla(220.0/360.0, 0.10, 0.85, 1.0) } }
pub fn text_primary() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.95, 1.0) } else { hsla(220.0/360.0, 0.15, 0.10, 1.0) } }
pub fn text_secondary() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.70, 1.0) } else { hsla(220.0/360.0, 0.10, 0.35, 1.0) } }
pub fn text_muted() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.50, 1.0) } else { hsla(220.0/360.0, 0.05, 0.55, 1.0) } }
pub fn accent() -> Hsla { hsla(239.0 / 360.0, 0.84, if is_dark() { 0.67 } else { 0.55 }, 1.0) }
pub fn danger() -> Hsla { hsla(0.0, 0.75, if is_dark() { 0.55 } else { 0.45 }, 1.0) }
pub fn success() -> Hsla { hsla(150.0 / 360.0, 0.65, if is_dark() { 0.45 } else { 0.35 }, 1.0) }

pub fn accent_bg() -> Hsla { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) }
pub fn ink_white() -> Hsla { hsla(0.0, 0.0, 1.0, 1.0) }
pub fn transparent() -> Hsla { hsla(0.0, 0.0, 0.0, 0.0) }

pub fn hex_to_hsla(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    let max = r.max(g).max(b); let min = r.min(g).min(b); let l = (max + min) / 2.0;
    if (max - min).abs() < 0.001 { return hsla(0.0, 0.0, l, 1.0); }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 0.001 { (g - b) / d + if g < b { 6.0 } else { 0.0 } }
        else if (max - g).abs() < 0.001 { (b - r) / d + 2.0 }
        else { (r - g) / d + 4.0 } / 6.0;
    hsla(h, s, l, 1.0)
}

pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' { in_escape = true; continue; }
        if in_escape { if ch.is_ascii_alphabetic() { in_escape = false; } continue; }
        if ch == '\r' { continue; }
        result.push(ch);
    }
    result
}
