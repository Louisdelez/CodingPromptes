#![allow(dead_code)]
use gpui::*;

pub struct InkwellTheme {
    pub bg_primary: Hsla,
    pub bg_secondary: Hsla,
    pub bg_tertiary: Hsla,
    pub bg_hover: Hsla,
    pub border: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub danger: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub terminal_bg: Hsla,
    pub terminal_text: Hsla,
}

impl InkwellTheme {
    pub fn dark() -> Self {
        use crate::ui::colors::hex_to_hsla;
        Self {
            bg_primary: hex_to_hsla("0f1117"),
            bg_secondary: hex_to_hsla("1a1b23"),
            bg_tertiary: hex_to_hsla("22232d"),
            bg_hover: hex_to_hsla("2a2b37"),
            border: hex_to_hsla("2e303a"),
            text_primary: hex_to_hsla("f3f4f6"),
            text_secondary: hex_to_hsla("9ca3af"),
            text_muted: hex_to_hsla("6b7280"),
            accent: hex_to_hsla("6366f1"),
            accent_hover: hex_to_hsla("818cf8"),
            danger: hex_to_hsla("f87171"),
            success: hex_to_hsla("34d399"),
            warning: hex_to_hsla("fbbf24"),
            terminal_bg: hsla(0.0, 0.0, 0.04, 1.0),
            terminal_text: hsla(120.0 / 360.0, 0.8, 0.6, 1.0),
        }
    }

    pub fn light() -> Self {
        use crate::ui::colors::hex_to_hsla;
        Self {
            bg_primary: hsla(0.0, 0.0, 1.0, 1.0),
            bg_secondary: hex_to_hsla("f8f9fa"),
            bg_tertiary: hex_to_hsla("f0f1f3"),
            bg_hover: hex_to_hsla("e8e9ed"),
            border: hex_to_hsla("d1d5db"),
            text_primary: hex_to_hsla("111827"),
            text_secondary: hex_to_hsla("4b5563"),
            text_muted: hex_to_hsla("9ca3af"),
            accent: hex_to_hsla("4f46e5"),
            accent_hover: hex_to_hsla("6366f1"),
            danger: hex_to_hsla("dc2626"),
            success: hex_to_hsla("059669"),
            warning: hex_to_hsla("d97706"),
            terminal_bg: hex_to_hsla("f5f5f5"),
            terminal_text: hex_to_hsla("1f2937"),
        }
    }

    pub fn from_mode(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }
}
