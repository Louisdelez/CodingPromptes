use gpui::*;

pub struct InkwellTheme {
    pub bg_primary: Hsla,
    pub bg_secondary: Hsla,
    pub bg_tertiary: Hsla,
    pub border: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub accent: Hsla,
    pub danger: Hsla,
    pub success: Hsla,
    pub terminal_bg: Hsla,
    pub terminal_text: Hsla,
}

impl InkwellTheme {
    pub fn dark() -> Self {
        Self {
            bg_primary: hsla(230.0 / 360.0, 0.15, 0.07, 1.0),
            bg_secondary: hsla(230.0 / 360.0, 0.12, 0.10, 1.0),
            bg_tertiary: hsla(230.0 / 360.0, 0.10, 0.14, 1.0),
            border: hsla(230.0 / 360.0, 0.10, 0.20, 1.0),
            text_primary: hsla(0.0, 0.0, 0.95, 1.0),
            text_secondary: hsla(0.0, 0.0, 0.70, 1.0),
            text_muted: hsla(0.0, 0.0, 0.50, 1.0),
            accent: hsla(239.0 / 360.0, 0.84, 0.67, 1.0),
            danger: hsla(0.0, 0.75, 0.55, 1.0),
            success: hsla(150.0 / 360.0, 0.65, 0.45, 1.0),
            terminal_bg: hsla(0.0, 0.0, 0.04, 1.0),
            terminal_text: hsla(120.0 / 360.0, 0.8, 0.6, 1.0),
        }
    }

    pub fn light() -> Self {
        Self {
            bg_primary: hsla(0.0, 0.0, 1.0, 1.0),
            bg_secondary: hsla(220.0 / 360.0, 0.10, 0.97, 1.0),
            bg_tertiary: hsla(220.0 / 360.0, 0.08, 0.93, 1.0),
            border: hsla(220.0 / 360.0, 0.10, 0.85, 1.0),
            text_primary: hsla(220.0 / 360.0, 0.15, 0.10, 1.0),
            text_secondary: hsla(220.0 / 360.0, 0.10, 0.35, 1.0),
            text_muted: hsla(220.0 / 360.0, 0.05, 0.55, 1.0),
            accent: hsla(239.0 / 360.0, 0.84, 0.55, 1.0),
            danger: hsla(0.0, 0.80, 0.45, 1.0),
            success: hsla(150.0 / 360.0, 0.70, 0.35, 1.0),
            terminal_bg: hsla(220.0 / 360.0, 0.05, 0.95, 1.0),
            terminal_text: hsla(220.0 / 360.0, 0.15, 0.20, 1.0),
        }
    }

    pub fn from_mode(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }
}
