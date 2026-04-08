use gpui::*;

fn bg_primary() -> Hsla { hsla(230.0 / 360.0, 0.15, 0.07, 1.0) }
fn bg_secondary() -> Hsla { hsla(230.0 / 360.0, 0.12, 0.10, 1.0) }
fn border_color() -> Hsla { hsla(230.0 / 360.0, 0.10, 0.20, 1.0) }
fn text_primary() -> Hsla { hsla(0.0, 0.0, 0.95, 1.0) }
fn text_secondary() -> Hsla { hsla(0.0, 0.0, 0.70, 1.0) }
fn text_muted() -> Hsla { hsla(0.0, 0.0, 0.50, 1.0) }
fn accent() -> Hsla { hsla(239.0 / 360.0, 0.84, 0.67, 1.0) }

pub struct InkwellApp {
    screen: Screen,
}

#[derive(Clone, Copy, PartialEq)]
enum Screen { Auth, Ide }

impl InkwellApp {
    pub fn new() -> Self {
        Self { screen: Screen::Auth }
    }
}

impl Render for InkwellApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.screen {
            Screen::Auth => self.render_auth(cx),
            Screen::Ide => self.render_ide(cx),
        }
    }
}

impl InkwellApp {
    fn render_auth(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .size_full()
            .bg(bg_primary())
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(380.0))
                    .p(px(32.0))
                    .bg(bg_secondary())
                    .rounded(px(16.0))
                    .border_1()
                    .border_color(border_color())
                    .flex()
                    .flex_col()
                    .gap(px(20.0))
                    .child(
                        div().flex().flex_col().items_center().gap(px(8.0))
                            .child(div().text_xl().text_color(text_primary()).child("Inkwell"))
                            .child(div().text_sm().text_color(text_muted()).child("GPU-Accelerated Prompt IDE"))
                    )
                    .child(
                        div().text_sm().text_color(text_secondary()).child("Click to enter IDE")
                    )
            )
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _cx| {
                this.screen = Screen::Ide;
            }))
    }

    fn render_ide(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .size_full()
            .bg(bg_primary())
            .flex()
            .flex_col()
            // Header
            .child(
                div().h(px(40.0)).px(px(16.0)).flex().items_center().gap(px(12.0))
                    .border_b_1().border_color(border_color()).bg(bg_secondary())
                    .child(div().text_sm().text_color(accent()).child("Inkwell"))
                    .child(div().w(px(1.0)).h(px(16.0)).bg(border_color()))
                    .child(div().text_sm().text_color(text_secondary()).child("New prompt"))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(text_muted()).child("GPUI Native"))
            )
            // Main
            .child(
                div().flex_1().flex().overflow_hidden()
                    // Sidebar
                    .child(
                        div().w(px(250.0)).flex_shrink_0().border_r_1().border_color(border_color()).bg(bg_secondary())
                            .flex().flex_col()
                            .child(
                                div().h(px(36.0)).px(px(12.0)).flex().items_center().border_b_1().border_color(border_color())
                                    .child(div().text_xs().text_color(accent()).child("Library"))
                            )
                            .child(div().flex_1().p(px(12.0)).child(div().text_xs().text_color(text_muted()).child("Projects...")))
                    )
                    // Editor
                    .child(
                        div().flex_1().flex().flex_col().min_w_0().overflow_hidden()
                            .child(
                                div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0))
                                    .child(render_block("Constitution", "#a78bfa"))
                                    .child(render_block("Specification", "#60a5fa"))
                                    .child(render_block("Plan", "#34d399"))
                                    .child(render_block("Tasks", "#fbbf24"))
                                    .child(render_block("Implementation", "#f87171"))
                                    .child(
                                        div().py(px(12.0)).flex().items_center().justify_center()
                                            .rounded(px(8.0)).border_1().border_color(border_color())
                                            .text_sm().text_color(text_muted()).child("+ Add block")
                                    )
                            )
                    )
                    // Right panel
                    .child(
                        div().w(px(380.0)).flex_shrink_0().border_l_1().border_color(border_color()).bg(bg_secondary())
                            .flex().flex_col()
                            .child(
                                div().h(px(36.0)).px(px(12.0)).flex().items_center().gap(px(8.0)).border_b_1().border_color(border_color())
                                    .child(div().text_xs().text_color(accent()).child("Preview"))
                                    .child(div().text_xs().text_color(text_muted()).child("Playground"))
                                    .child(div().text_xs().text_color(text_muted()).child("Terminal"))
                            )
                            .child(div().flex_1().p(px(12.0)).child(div().text_xs().text_color(text_muted()).child("Panel content...")))
                    )
            )
            // Bottom bar
            .child(
                div().h(px(28.0)).px(px(12.0)).flex().items_center().gap(px(12.0))
                    .border_t_1().border_color(border_color()).bg(bg_secondary())
                    .child(div().text_xs().text_color(text_muted()).child("0 tokens"))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(text_muted()).child("Terminal"))
            )
    }
}

fn render_block(label: &str, hex: &str) -> Div {
    let color = hex_to_hsla(hex);
    div()
        .rounded(px(8.0)).border_1().border_color(border_color()).bg(bg_secondary()).overflow_hidden()
        .child(
            div().px(px(12.0)).py(px(8.0)).flex().items_center().gap(px(8.0)).border_b_1().border_color(border_color())
                .child(div().w(px(3.0)).h(px(14.0)).rounded(px(2.0)).bg(color))
                .child(div().text_sm().text_color(color).child(label.to_string()))
        )
        .child(
            div().p(px(12.0)).min_h(px(60.0)).text_sm().text_color(text_muted())
                .child(format!("# {label}\n\nEdit here..."))
        )
}

fn hex_to_hsla(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 0.001 { return hsla(0.0, 0.0, l, 1.0); }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 0.001 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 0.001 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    hsla(h, s, l, 1.0)
}
