#![allow(dead_code)]
use gpui::*;
use gpui_platform::application;
use gpui_component::Root;
use gpui_component_assets::Assets;

mod app;
mod components;
mod dock;
mod kiro;
mod layout;
mod llm;
mod panels;
mod persistence;
mod settings;
mod spec;
mod state;
mod store;
mod theme;
mod types;
mod ui;
mod devtools;

struct DevToolsLogger {
    inner: env_logger::Logger,
}

impl log::Log for DevToolsLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.inner.log(record);
            devtools::push_log(format!(
                "[{}] {} — {}",
                record.level(),
                record.target(),
                record.args()
            ));
        }
    }
    fn flush(&self) { self.inner.flush(); }
}

fn init_logging() {
    let inner = env_logger::Builder::from_default_env().build();
    let level = inner.filter();
    let logger = DevToolsLogger { inner };
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(level);
    }
}

fn main() {
    init_logging();

    application().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        // Start in dark mode (matches our default)
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);

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
            KeyBinding::new("cmd-z", app::Undo, None),
            KeyBinding::new("ctrl-z", app::Undo, None),
            KeyBinding::new("cmd-s", app::SaveNow, None),
            KeyBinding::new("ctrl-s", app::SaveNow, None),
            KeyBinding::new("ctrl-tab", app::FocusNextPanel, None),
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
                let app_view: Entity<app::InkwellApp> = cx.new(|cx| app::InkwellApp::new(window, cx));
                let any_view: AnyView = app_view.into();
                cx.new(|cx| Root::new(any_view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
