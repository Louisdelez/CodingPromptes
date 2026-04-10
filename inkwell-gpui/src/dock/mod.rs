//! DockArea — manages the three-panel layout with resize handles.
//! Inspired by Zed's Dock system: left dock | center | right dock.

use gpui::*;
use crate::store::{AppStore, StoreEvent};
use crate::ui::colors::*;

/// Drag type for left panel resize handle
#[derive(Clone)]
pub struct LeftResizeDrag;

/// Drag type for right panel resize handle
#[derive(Clone)]
pub struct RightResizeDrag;

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

/// DockArea manages a three-panel layout: left | center | right
/// with draggable resize handles between panels.
pub struct DockArea {
    store: Entity<AppStore>,
    left: Option<AnyView>,
    center: AnyView,
    right: Option<AnyView>,
}

impl DockArea {
    pub fn new(
        store: Entity<AppStore>,
        center: AnyView,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.subscribe(&store, |_, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::SettingsChanged => cx.notify(),
                _ => {}
            }
        }).detach();

        Self { store, left: None, center, right: None }
    }

    pub fn set_left(&mut self, view: AnyView) {
        self.left = Some(view);
    }

    pub fn set_right(&mut self, view: AnyView) {
        self.right = Some(view);
    }
}

impl Render for DockArea {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.store.read(cx);
        let left_open = s.left_open;
        let right_open = s.right_open;

        let mut row = div().flex_1().flex().overflow_hidden();

        // Left panel + resize handle
        if left_open {
            if let Some(ref left) = self.left {
                row = row.child(left.clone());
                row = row.child(
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
        }

        // Center (always visible)
        row = row.child(self.center.clone());

        // Right panel + resize handle
        if right_open {
            if let Some(ref right) = self.right {
                row = row.child(
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
                row = row.child(right.clone());
            }
        }

        row
    }
}
