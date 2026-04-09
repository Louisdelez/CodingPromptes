use gpui::*;
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct BottomBar {
    store: Entity<AppStore>,
}

impl BottomBar {
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.subscribe(&store, |_this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::PromptCacheUpdated | StoreEvent::ProjectChanged => {
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        Self { store }
    }
}

impl Render for BottomBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let chars = store.cached_chars;
        let words = store.cached_words;
        let lines = store.cached_lines;
        let tokens = store.cached_tokens;
        let cost = tokens as f64 * 0.000003;
        let enabled = store.project.blocks.iter().filter(|b| b.enabled).count();
        let total = store.project.blocks.len();
        let model = store.selected_model.clone();
        drop(store);

        div().h(px(28.0)).px(px(12.0)).flex().items_center().gap(px(10.0))
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .child(div().text_xs().text_color(text_muted()).child(format!("{chars} car.")))
            .child(div().text_xs().text_color(text_muted()).child(format!("{words} mots")))
            .child(div().text_xs().text_color(text_muted()).child(format!("{lines} lignes")))
            .child(div().text_xs().text_color(text_muted()).child(format!("~{tokens} tokens")))
            .child(div().text_xs().text_color(text_muted()).child(format!("~${cost:.6}")))
            .child({
                let max_ctx = 128000u64;
                let pct = (tokens as f64 / max_ctx as f64 * 100.0).min(100.0);
                let bar_color = if pct > 80.0 { danger() } else if pct > 50.0 { hsla(50.0 / 360.0, 0.8, 0.5, 1.0) } else { accent() };
                div().w(px(40.0)).h(px(4.0)).rounded(px(2.0)).bg(bg_tertiary())
                    .child(div().h(px(4.0)).rounded(px(2.0)).bg(bar_color)
                        .w(px(pct as f32 / 100.0 * 40.0)))
            })
            .child(div().text_xs().text_color(text_muted()).child(format!("{:.1}%", tokens as f64 / 128000.0 * 100.0)))
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_muted()).child(format!("{enabled}/{total} blocs")))
            .child(div().flex_1())
            // Terminal button
            .child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted()).child(Icon::new(IconName::SquareTerminal)).child("Terminal")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.store.update(cx, |s, cx| {
                            s.right_tab = RightTab::Terminal;
                            s.right_open = true;
                            cx.emit(StoreEvent::SwitchRightTab(RightTab::Terminal));
                        });
                    }))
            )
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_secondary()).child(model))
    }
}
