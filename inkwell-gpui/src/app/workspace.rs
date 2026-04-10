use gpui::*;
use gpui_component::{Theme, ThemeMode};
use crate::ui::colors::*;
use crate::state::*;

use super::{InkwellApp, NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow};

/// Separate drag types so left/right resize handles don't interfere
#[derive(Clone)]
pub(crate) struct LeftResizeDrag;

#[derive(Clone)]
pub(crate) struct RightResizeDrag;

impl Render for LeftResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(4.0)).h(px(40.0)).bg(accent()).rounded(px(2.0))
    }
}

impl Render for RightResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(4.0)).h(px(40.0)).bg(accent()).rounded(px(2.0))
    }
}

impl InkwellApp {
    pub(crate) fn render_ide(&mut self, cx: &mut Context<Self>) -> Div {
        // Read layout state from store (not self.state) to avoid bridge re-renders
        let s = self.store.read(cx);
        let left_open = s.left_open;
        let right_open = s.right_open;
        let show_settings = s.show_settings;
        let show_profile = s.show_profile;
        let dark_mode = s.dark_mode;
        let left_w = s.left_width;
        let right_w = s.right_width;

        set_dark_mode(dark_mode);
        // Sync gpui-component theme so Input, Button, etc. follow dark/light mode
        Theme::change(if dark_mode { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);
        let t = crate::theme::InkwellTheme::from_mode(dark_mode);
        let mut main_row = div().flex_1().flex().overflow_hidden();
        // left_w and right_w already read above
        if left_open {
            main_row = main_row.child(self.left_panel.clone());
            // Left resize handle — only reacts to LeftResizeDrag
            main_row = main_row.child(
                div().id("left-resize").w(px(4.0)).flex_shrink_0().cursor_pointer()
                    .hover(|s| s.bg(accent()))
                    .on_drag(LeftResizeDrag, |drag, _, _, cx| cx.new(|_| drag.clone()))
                    .on_drag_move(cx.listener(|this, ev: &DragMoveEvent<LeftResizeDrag>, _, cx| {
                        let new_w = f32::from(ev.event.position.x).clamp(180.0, 500.0);
                        this.store.update(cx, |s, _| { s.left_width = new_w; });
                        cx.notify();
                    }))
            );
        }
        main_row = main_row.child(self.editor.clone());
        if right_open {
            // Right resize handle — only reacts to RightResizeDrag
            main_row = main_row.child(
                div().id("right-resize").w(px(4.0)).flex_shrink_0().cursor_pointer()
                    .hover(|s| s.bg(accent()))
                    .on_drag(RightResizeDrag, |drag, _, _, cx| cx.new(|_| drag.clone()))
                    .on_drag_move(cx.listener(|this, ev: &DragMoveEvent<RightResizeDrag>, window, cx| {
                        let mouse_x = f32::from(ev.event.position.x);
                        let win_w = f32::from(window.viewport_size().width);
                        let new_w = (win_w - mouse_x).clamp(250.0, 600.0);
                        this.store.update(cx, |s, _| { s.right_width = new_w; });
                        cx.notify();
                    }))
            );
            main_row = main_row.child(self.right_panel.clone());
        }

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
                this.state.save_timer = 1; // Save next frame
            }))
            .child(self.header.clone()) // Entity<HeaderBar> — only re-renders when store emits relevant event
            .child(main_row)
            .children(if show_settings { Some(self.render_settings(cx)) } else { None })
            .children(if show_profile { Some(self.render_profile(cx)) } else { None })
            .child(self.bottom_bar.clone()) // Entity<BottomBar> — only re-renders on PromptCacheUpdated
    }



}
