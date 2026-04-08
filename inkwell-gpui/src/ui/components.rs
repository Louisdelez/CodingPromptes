use gpui::*;
use super::colors::*;

/// Tab button with active state
pub fn tab_button(label: &str, active: bool) -> Div {
    div()
        .px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
        .cursor_pointer()
        .text_color(if active { accent() } else { text_muted() })
        .bg(if active { accent_bg() } else { transparent() })
        .hover(|s| s.bg(if active { accent_bg() } else { hsla(0.0, 0.0, 0.5, 0.05) }))
        .child(label.to_string())
}

/// Action button (small, for toolbars)
pub fn action_button(label: &str, color: Hsla) -> Div {
    div()
        .px(px(6.0)).py(px(3.0)).rounded(px(4.0))
        .cursor_pointer()
        .text_xs().text_color(color)
        .hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.08)))
        .child(label.to_string())
}

/// Primary button (filled)
pub fn primary_button(label: &str, loading: bool) -> Div {
    div()
        .py(px(8.0)).px(px(16.0))
        .rounded(px(8.0))
        .bg(if loading { text_muted() } else { accent() })
        .cursor_pointer()
        .flex().items_center().justify_center()
        .text_sm().text_color(white())
        .hover(|s| s.opacity(0.9))
        .child(label.to_string())
}

/// Section header
pub fn section_header(label: &str) -> Div {
    div()
        .text_xs().text_color(text_muted())
        .child(label.to_string())
}

/// Separator line
pub fn separator() -> Div {
    div().h(px(1.0)).bg(border_c())
}

/// Badge (small label)
pub fn badge(label: &str, color: Hsla) -> Div {
    div()
        .px(px(6.0)).py(px(2.0)).rounded(px(4.0))
        .bg(hsla(color.h, color.s, color.l, 0.1))
        .text_xs().text_color(color)
        .child(label.to_string())
}

/// Card container
pub fn card() -> Div {
    div()
        .rounded(px(8.0)).border_1().border_color(border_c())
        .bg(bg_secondary()).overflow_hidden()
}

/// Status dot
pub fn status_dot(online: bool) -> Div {
    div()
        .w(px(6.0)).h(px(6.0)).rounded(px(3.0))
        .bg(if online { success() } else { text_muted() })
}
