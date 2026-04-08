use gpui::*;
use gpui_platform::application;
use gpui_component::Root;

mod app;
mod state;
mod theme;

fn main() {
    env_logger::init();

    application().run(|cx: &mut App| {
        gpui_component::init(cx);

        // Register keyboard shortcuts
        cx.bind_keys([
            KeyBinding::new("cmd-n", app::NewProject, None),
            KeyBinding::new("ctrl-n", app::NewProject, None),
            KeyBinding::new("cmd-`", app::ToggleTerminal, None),
            KeyBinding::new("ctrl-`", app::ToggleTerminal, None),
            KeyBinding::new("cmd-enter", app::RunPrompt, None),
            KeyBinding::new("ctrl-enter", app::RunPrompt, None),
            KeyBinding::new("cmd-,", app::ToggleSettings, None),
            KeyBinding::new("ctrl-,", app::ToggleSettings, None),
        ]);
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
