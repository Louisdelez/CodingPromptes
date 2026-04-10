#![allow(dead_code)]
use gpui::*;
use std::cell::RefCell;

thread_local! {
    static DARK_MODE: RefCell<bool> = const { RefCell::new(true) };
}

pub fn set_dark_mode(dark: bool) { DARK_MODE.with(|d| *d.borrow_mut() = dark); }
pub fn is_dark() -> bool { DARK_MODE.with(|d| *d.borrow()) }

// Dark: #0f1117, Light: #ffffff
pub fn bg_primary() -> Hsla { if is_dark() { hex_to_hsla("0f1117") } else { hsla(0.0, 0.0, 1.0, 1.0) } }
// Dark: #1a1b23, Light: #f8f9fa
pub fn bg_secondary() -> Hsla { if is_dark() { hex_to_hsla("1a1b23") } else { hex_to_hsla("f8f9fa") } }
// Dark: #22232d, Light: #f0f1f3
pub fn bg_tertiary() -> Hsla { if is_dark() { hex_to_hsla("22232d") } else { hex_to_hsla("f0f1f3") } }
// Dark: #2a2b37 (hover)
pub fn bg_hover() -> Hsla { if is_dark() { hex_to_hsla("2a2b37") } else { hex_to_hsla("e8e9ed") } }
// Dark: #2e303a, Light: #d1d5db
pub fn border_c() -> Hsla { if is_dark() { hex_to_hsla("2e303a") } else { hex_to_hsla("d1d5db") } }
// Dark: #f3f4f6, Light: #111827
pub fn text_primary() -> Hsla { if is_dark() { hex_to_hsla("f3f4f6") } else { hex_to_hsla("111827") } }
// Dark: #9ca3af, Light: #4b5563
pub fn text_secondary() -> Hsla { if is_dark() { hex_to_hsla("9ca3af") } else { hex_to_hsla("4b5563") } }
// Dark: #6b7280, Light: #9ca3af
pub fn text_muted() -> Hsla { if is_dark() { hex_to_hsla("6b7280") } else { hex_to_hsla("9ca3af") } }
// Dark: #6366f1, Light: #4f46e5
pub fn accent() -> Hsla { if is_dark() { hex_to_hsla("6366f1") } else { hex_to_hsla("4f46e5") } }
// Dark: #818cf8 (hover accent)
pub fn accent_hover() -> Hsla { if is_dark() { hex_to_hsla("818cf8") } else { hex_to_hsla("6366f1") } }
// Dark: #f87171, Light: #dc2626
pub fn danger() -> Hsla { if is_dark() { hex_to_hsla("f87171") } else { hex_to_hsla("dc2626") } }
// Dark: #34d399, Light: #059669
pub fn success() -> Hsla { if is_dark() { hex_to_hsla("34d399") } else { hex_to_hsla("059669") } }
// Dark: #fbbf24, Light: #d97706
pub fn warning() -> Hsla { if is_dark() { hex_to_hsla("fbbf24") } else { hex_to_hsla("d97706") } }

pub fn accent_bg() -> Hsla { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) }
pub fn ink_white() -> Hsla { hsla(0.0, 0.0, 1.0, 1.0) }
pub fn transparent() -> Hsla { hsla(0.0, 0.0, 0.0, 0.0) }

pub fn hex_to_hsla(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 { return hsla(0.0, 0.0, 0.5, 1.0); } // Fallback for invalid hex
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
