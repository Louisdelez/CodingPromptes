use gpui::*;
use gpui_component::{Theme, ThemeMode};
use crate::ui::colors::*;
use crate::state::*;

use super::{InkwellApp, NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow};

// Re-export drag types from dock for backward compat
pub(crate) use crate::dock::{LeftResizeDrag, RightResizeDrag};

impl InkwellApp {
    pub(crate) fn render_ide(&mut self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let show_settings = s.show_settings;
        let show_profile = s.show_profile;
        let dark_mode = s.dark_mode;

        set_dark_mode(dark_mode);
        Theme::change(if dark_mode { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);
        let t = crate::theme::InkwellTheme::from_mode(dark_mode);

        // DockArea handles the three-panel layout + resize handles
        let dock = self.dock.clone();

        div().size_full().bg(t.bg_primary).flex().flex_col()
            .on_action(cx.listener(|this, _: &NewProject, _, _| {
                let mut p = Project::default_prompt();
                p.name = "Nouveau Prompte".into();
                let now = chrono::Local::now();
                p.tags.push(now.format("%Y-%m-%d %H:%M").to_string());
                this.state.project = p;
                this.state.block_inputs.clear();
            }))
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, cx| {
                this.state.right_tab = RightTab::Terminal;
                this.state.right_open = true;
                this.store.update(cx, |s, cx| { s.right_tab = RightTab::Terminal; s.right_open = true; cx.emit(crate::store::StoreEvent::SwitchRightTab(RightTab::Terminal)); });
            }))
            .on_action(cx.listener(|this, _: &RunPrompt, _, cx| {
                this.state.right_tab = RightTab::Playground;
                this.state.right_open = true;
                this.store.update(cx, |s, cx| { s.right_tab = RightTab::Playground; s.right_open = true; cx.emit(crate::store::StoreEvent::SwitchRightTab(RightTab::Playground)); });
            }))
            .on_action(cx.listener(|this, _: &ToggleSettings, _, cx| {
                this.state.show_settings = !this.state.show_settings;
                this.store.update(cx, |s, cx| { s.show_settings = !s.show_settings; cx.emit(crate::store::StoreEvent::SettingsChanged); });
            }))
            .on_action(cx.listener(|this, _: &Undo, _, _| {
                if let Some(prev_blocks) = this.state.undo_stack.pop_back() {
                    this.state.project.blocks = prev_blocks;
                    this.state.block_inputs.clear();
                }
            }))
            .on_action(cx.listener(|this, _: &SaveNow, _, _| {
                this.state.save_pending = true;
                this.state.save_timer = 1;
            }))
            .child(self.header.clone())
            .child(dock)
            .children(if show_settings { Some(self.render_settings(cx)) } else { None })
            .children(if show_profile { Some(self.render_profile(cx)) } else { None })
            .child(self.bottom_bar.clone())
    }
}
