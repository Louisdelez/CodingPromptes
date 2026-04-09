use gpui::*;
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

fn stat_item(icon: IconName, value: &str, label: &str) -> Div {
    let mut d = div().flex().items_center().gap(px(3.0))
        .child(Icon::new(icon).text_color(text_muted()))
        .child(div().text_xs().text_color(text_muted()).child(value.to_string()));
    if !label.is_empty() {
        d = d.child(div().text_xs().text_color(text_muted()).child(label.to_string()));
    }
    d
}

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

        let max_ctx = 128000u64;
        let pct = (tokens as f64 / max_ctx as f64 * 100.0).min(100.0);
        let bar_color = if pct > 80.0 { danger() } else if pct > 50.0 { warning() } else { accent() };

        div().h(px(32.0)).px(px(12.0)).flex().items_center().gap(px(12.0))
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            // Stats with icons (matching web: T chars, wrap words, lines, # tokens, coins cost, zap blocks)
            .child(stat_item(IconName::Type, &format!("{chars}"), "car."))
            .child(stat_item(IconName::WrapText, &format!("{words}"), "mots"))
            .child(stat_item(IconName::AlignLeft, &format!("{lines}"), "lignes"))
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(stat_item(IconName::Hash, &format!("{tokens}"), "tokens"))
            .child(stat_item(IconName::Coins, &format!("~${cost:.6}"), ""))
            .child(stat_item(IconName::Zap, &format!("{enabled}/{total}"), "blocs"))
            .child(div().flex_1())
            // Terminal button (matching web)
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).flex().items_center().gap(px(4.0))
                .text_xs().text_color(text_muted()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(IconName::SquareTerminal))
                .child("Terminal")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        s.right_tab = RightTab::Terminal;
                        s.right_open = true;
                        cx.emit(StoreEvent::SwitchRightTab(RightTab::Terminal));
                    });
                })))
            // Context usage bar
            .child(div().w(px(60.0)).h(px(4.0)).rounded(px(2.0)).bg(bg_tertiary())
                .child(div().h(px(4.0)).rounded(px(2.0)).bg(bar_color)
                    .w(px(pct as f32 / 100.0 * 60.0))))
            .child(div().text_xs().text_color(text_muted()).child(format!("{:.1}%", pct)))
            // Model selector (with dropdown chevron like web)
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).border_1().border_color(border_c())
                .flex().items_center().gap(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(div().text_xs().text_color(text_secondary()).child(model))
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted())))
    }
}
