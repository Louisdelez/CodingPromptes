mod tabs;
use gpui::*;
use gpui_component::input::InputState;
use gpui_component::{Icon, IconName};
use gpui_component::animation::cubic_bezier;
use std::time::Duration;
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct RightPanel {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) store: Entity<AppStore>,
    pub(crate) active_tab: RightTab,
    pub(crate) show_dropdown: bool,
    pub(crate) chat_input: Option<Entity<InputState>>,
    pub(crate) copy_feedback_at: Option<std::time::Instant>,
}

impl Focusable for RightPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

impl RightPanel {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let chat_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Envoyer un message...")));
        cx.subscribe(&store, |this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::PlaygroundUpdated | StoreEvent::PromptCacheUpdated |
                StoreEvent::ChatMessageReceived | StoreEvent::TerminalOutput |
                StoreEvent::ProjectChanged => cx.notify(),
                StoreEvent::SwitchRightTab(tab) => { this.active_tab = *tab; cx.notify(); }
                StoreEvent::CloseAllMenus => { this.show_dropdown = false; cx.notify(); }
                _ => {}
            }
        }).detach();
        Self { focus_handle, store, active_tab: RightTab::Preview, show_dropdown: false, chat_input, copy_feedback_at: None }
    }
}

impl Render for RightPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        const TABS: &[(&str, RightTab, IconName)] = &[
            ("Preview", RightTab::Preview, IconName::File),
            ("Playground", RightTab::Playground, IconName::Play),
            ("Historique", RightTab::History, IconName::Clock),
            ("STT", RightTab::Stt, IconName::Mic),
            ("IA", RightTab::Optimize, IconName::Sparkles),
            ("Lint", RightTab::Lint, IconName::TriangleAlert),
            ("Export", RightTab::Export, IconName::Download),
            ("Stats", RightTab::Analytics, IconName::ChartPie),
            ("Chain", RightTab::Chain, IconName::Network),
            ("Chat", RightTab::Chat, IconName::Bot),
            ("Collab", RightTab::Collab, IconName::User),
            ("GPU", RightTab::Fleet, IconName::Globe),
            ("SDD", RightTab::Sdd, IconName::Scroll),
        ];

        let active = self.active_tab;
        let (tab_label, tab_icon) = TABS.iter()
            .find(|(_, t, _)| *t == active)
            .map(|(l, _, i)| (*l, i.clone()))
            .unwrap_or(("Preview", IconName::File));
        let show_dd = self.show_dropdown;

        // Header — full button click triggers dropdown
        let header = div().id("right-panel-header").h(px(44.0)).px(px(16.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c())
            .cursor_pointer().hover(|s| s.bg(bg_hover()))
            .child(Icon::new(tab_icon).text_color(text_muted()))
            .child(div().flex_1().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child(tab_label))
            .child(Icon::new(if show_dd { IconName::ChevronUp } else { IconName::ChevronDown }).text_color(text_muted()))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.show_dropdown = !this.show_dropdown; cx.notify();
            }));

        // Dropdown — hover animations on items
        let panel_width = self.store.read(cx).right_width;
        let dropdown = if show_dd {
            let mut menu = div().mx(px(8.0)).mt(px(4.0)).rounded(px(8.0))
                .bg(bg_secondary()).border_1().border_color(border_c()).p(px(4.0)).flex().flex_col().gap(px(2.0))
                .w(px(panel_width - 16.0));
            for (label, tab, icon) in TABS {
                let tab = *tab; let icon = icon.clone();
                let is_active = self.active_tab == tab;
                menu = menu.child(div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).flex().items_center().gap(px(8.0))
                    .text_sm().cursor_pointer()
                    .text_color(if is_active { accent() } else { text_primary() })
                    .bg(if is_active { accent_bg() } else { transparent() })
                    .hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(icon)).child(label.to_string())
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.active_tab = tab; this.show_dropdown = false;
                        this.store.update(cx, |s, _| { s.right_tab = tab; }); cx.notify();
                    })));
            }
            // Floating overlay
            Some(deferred(
                anchored().snap_to_window_with_margin(px(8.0)).child(menu)
            ).with_priority(1))
        } else { None };

        let content = match self.active_tab {
            RightTab::Preview => self.tab_preview(cx),
            RightTab::Playground => self.tab_playground(cx),
            RightTab::Chat => self.tab_chat(cx),
            RightTab::Stt => self.tab_stt(cx),
            RightTab::Optimize => self.tab_optimize(cx),
            RightTab::Lint => self.tab_lint(cx),
            RightTab::Fleet => self.tab_fleet(cx),
            RightTab::Terminal => self.tab_terminal(cx),
            RightTab::Export => self.tab_export(cx),
            RightTab::History => self.tab_history(cx),
            RightTab::Analytics => self.tab_analytics(cx),
            RightTab::Chain => self.tab_chain(cx),
            RightTab::Collab => self.tab_collab(cx),
            RightTab::Sdd => self.tab_sdd(cx),
        };

        let anim = Animation::new(Duration::from_millis(150))
            .with_easing(cubic_bezier(0.25, 0.1, 0.25, 1.0));
        let content_animated = div().id("right-content").flex_1().overflow_y_scroll()
            .child(content)
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                if this.show_dropdown { this.show_dropdown = false; cx.notify(); }
            }))
            .with_animation(
                SharedString::from(format!("tab-fade-{:?}", active)),
                anim,
                |this, delta| this.opacity(delta),
            );

        let panel_width = self.store.read(cx).right_width;
        div().track_focus(&self.focus_handle).w(px(panel_width)).flex_shrink_0().border_l_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col().child(header).children(dropdown).child(content_animated)
    }
}

pub(super) fn lint(severity: &str, msg: &str) -> Div {
    let (color, icon) = match severity {
        "error" => (danger(), IconName::Close),
        "warning" => (hsla(50.0/360.0, 0.8, 0.5, 1.0), IconName::TriangleAlert),
        "success" => (success(), IconName::Check),
        _ => (text_muted(), IconName::Info),
    };
    div().px(px(10.0)).py(px(6.0)).rounded(px(6.0))
        .bg(hsla(color.h, color.s, color.l, 0.1)).border_1().border_color(hsla(color.h, color.s, color.l, 0.2))
        .flex().items_center().gap(px(8.0))
        .child(Icon::new(icon).text_color(color))
        .child(div().text_xs().text_color(color).child(msg.to_string()))
}

pub(super) fn kpi(label: &str, value: &str, color: Hsla) -> Div {
    div().flex_1().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
        .flex().flex_col().items_center().gap(px(4.0))
        .child(div().text_xl().text_color(color).child(value.to_string()))
        .child(div().text_xs().text_color(text_muted()).child(label.to_string()))
}

pub(super) fn export_btn(label: &str, desc: &str) -> Div {
    div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c()).bg(bg_tertiary())
        .flex().flex_col().gap(px(2.0))
        .child(div().flex().items_center().gap(px(6.0)).child(Icon::new(IconName::Download).text_color(text_muted()))
            .child(div().text_xs().text_color(text_secondary()).child(label.to_string())))
        .child(div().text_xs().text_color(text_muted()).child(desc.to_string()))
        .cursor_pointer().hover(|s| s.bg(hsla(239.0/360.0, 0.84, 0.67, 0.1)))
}

pub(super) fn time_btn(label: &str, active: bool) -> Div {
    div().px(px(10.0)).py(px(4.0)).rounded(px(6.0))
        .text_xs().cursor_pointer()
        .bg(if active { accent() } else { bg_tertiary() })
        .text_color(if active { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_secondary() })
        .border_1().border_color(if active { accent() } else { border_c() })
        .hover(|s| if active { s } else { s.bg(bg_secondary()) })
        .child(label.to_string())
}
