use gpui::*;
use gpui_component::{Icon, IconName};
use crate::ui::colors::*;

use super::InkwellApp;

impl InkwellApp {
    pub(crate) fn render_profile(&self, cx: &mut Context<Self>) -> Div {
        let session = self.store.read(cx).session.as_ref();
        let email = session.map(|s| s.email.clone()).unwrap_or_default();
        let display_name = session.map(|s| s.display_name.clone()).unwrap_or("User".into());
        let initial = email.chars().next().unwrap_or('U').to_uppercase().to_string();

        div().h(px(180.0)).flex_shrink_0()
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().text_color(text_primary()).child(Icon::new(IconName::User)).child("Profile"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(text_muted()).child(Icon::new(IconName::Close)).child("Close")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_profile = false;
                            }))
                    )
            )
            .child(
                div().flex().items_center().gap(px(16.0))
                    .child(
                        div().w(px(48.0)).h(px(48.0)).rounded(px(24.0)).bg(accent())
                            .flex().items_center().justify_center()
                            .text_xl().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(initial)
                    )
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_sm().text_color(text_primary()).child(display_name))
                        .child(div().text_xs().text_color(text_secondary()).child(email))
                        .child(div().text_xs().text_color(success()).child("Connected"))
                    )
            )
            .child(
                div().flex().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child(format!("Server: {}", self.store.read(cx).server_url)))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} projects", self.store.read(cx).projects.len())))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} workspaces", self.store.read(cx).workspaces.len())))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} executions", self.store.read(cx).executions.len())))
                )
    }
}
