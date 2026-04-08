use gpui::*;
use gpui_platform::application;

mod app;
mod state;

fn main() {
    env_logger::init();

    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Inkwell".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| app::InkwellApp::new())
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
