use gpui::*;
use gpui_platform::application;
use gpui_component::Root;

mod app;
mod state;

fn main() {
    env_logger::init();

    application().run(|cx: &mut App| {
        gpui_component::init(cx);
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
            |window, cx| {
                let app_view: Entity<app::InkwellApp> = cx.new(|_| app::InkwellApp::new());
                let any_view: AnyView = app_view.into();
                cx.new(|cx| Root::new(any_view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
