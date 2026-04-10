mod auth_screen;
mod workspace;
mod settings_modal;
mod profile_modal;
mod sync;

use gpui::*;
use gpui_component::input::InputState;
use crate::state::*;

// Re-export for use in sub-modules

// Actions for keyboard shortcuts
actions!(inkwell, [NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow, FocusNextPanel]);

// Global tokio runtime — reused by all async operations (avoids creating 25+ runtimes)
pub fn rt() -> &'static tokio::runtime::Runtime {
    use once_cell::sync::Lazy;
    static RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    });
    &RT
}

/// Build an LLM request with proper API routing and auth headers.
/// Routes to OpenAI/Anthropic/Google directly if API keys are set, else falls back to server proxy.
pub fn llm_post(client: &reqwest::Client, model: &str, server_url: &str, body: serde_json::Value) -> reqwest::RequestBuilder {
    let (ko, ka, kg, _) = crate::llm::load_local_keys();
    let (url, hdrs) = crate::llm::llm_endpoint(model, &ko, &ka, &kg, server_url);
    let msgs = body["messages"].as_array().cloned().unwrap_or_default();
    let rebuilt = crate::llm::build_llm_body(model, &msgs,
        body["temperature"].as_f64().unwrap_or(0.7) as f32,
        body["max_tokens"].as_u64().unwrap_or(4096) as u32,
        body["stream"].as_bool().unwrap_or(false),
    );
    let mut req = client.post(&url).json(&rebuilt);
    for (k, v) in &hdrs { req = req.header(k.as_str(), v.as_str()); }
    req
}

pub struct InkwellApp {
    pub state: AppState,
    pub store: Entity<crate::store::AppStore>,
    pub header: Entity<crate::components::header_bar::HeaderBar>,
    pub bottom_bar: Entity<crate::components::bottom_bar::BottomBar>,
    pub editor: Entity<crate::components::editor_pane::EditorPane>,
    pub left_panel: Entity<crate::panels::left::LeftPanel>,
    pub right_panel: Entity<crate::panels::right::RightPanel>,
    pub dock: Entity<crate::dock::DockArea>,
    pub auth_inputs: auth_screen::AuthScreenInputs,
    pub settings_inputs: settings_modal::SettingsInputs,
}

impl InkwellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (msg_tx, msg_rx) = std::sync::mpsc::channel();
        let store = cx.new(|_cx| crate::store::AppStore::new(msg_tx.clone()));
        let header = cx.new(|cx| crate::components::header_bar::HeaderBar::new(store.clone(), cx));
        let bottom_bar = cx.new(|cx| crate::components::bottom_bar::BottomBar::new(store.clone(), cx));
        let editor = cx.new(|cx| crate::components::editor_pane::EditorPane::new(store.clone(), window, cx));
        let left_panel = cx.new(|cx| crate::panels::left::LeftPanel::new(store.clone(), window, cx));
        let right_panel = cx.new(|cx| crate::panels::right::RightPanel::new(store.clone(), window, cx));

        // DockArea manages the three-panel layout + resize handles
        let lp: AnyView = left_panel.clone().into();
        let center: AnyView = editor.clone().into();
        let rp: AnyView = right_panel.clone().into();
        let dock = cx.new(|cx| {
            let mut d = crate::dock::DockArea::new(store.clone(), center, cx);
            d.set_left(lp);
            d.set_right(rp);
            d
        });

        let mut state = AppState::new_with_channel(msg_tx.clone(), msg_rx);
        state.dark_mode = store.read(cx).dark_mode;

        // Sync store changes back to state (temporary bridge during migration)
        cx.subscribe(&store, |this: &mut Self, _, event: &crate::store::StoreEvent, cx| {
            match event {
                crate::store::StoreEvent::SettingsChanged => {
                    let s = this.store.read(cx);
                    this.state.dark_mode = s.dark_mode;
                    this.state.lang = s.lang.clone();
                    this.state.show_settings = s.show_settings;
                    this.state.show_profile = s.show_profile;
                    this.state.left_open = s.left_open;
                    this.state.right_open = s.right_open;
                }
                crate::store::StoreEvent::ProjectChanged => {
                    let s = this.store.read(cx);
                    this.state.project.name = s.project.name.clone();
                    this.state.save_pending = s.save_pending;
                }
                crate::store::StoreEvent::SwitchRightTab(tab) => {
                    this.state.right_tab = *tab;
                    this.state.right_open = true;
                }
                _ => {}
            }
        }).detach();

        Self { state, store, header, bottom_bar, editor, left_panel, right_panel, dock, auth_inputs: auth_screen::AuthScreenInputs::default(), settings_inputs: settings_modal::SettingsInputs::default() }
    }

    fn t(&self) -> crate::theme::InkwellTheme {
        crate::theme::InkwellTheme::from_mode(self.state.dark_mode)
    }
}

impl InkwellApp {
    /// Start a periodic timer for background work (sync, timers, polling).
    /// This runs OUTSIDE of render — doesn't force re-renders.
    fn start_periodic_sync(cx: &mut Context<Self>) {
        // Run every 100ms (~10fps) instead of every frame (60fps)
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                let should_continue = this.update(cx, |this, cx| {
                    // Poll async messages
                    this.poll_messages(cx);

                    // Sync editor content to store
                    let changed = this.editor.update(cx, |e, cx| e.sync_content(cx));
                    if changed {
                        this.store.update(cx, |s, cx| {
                            if s.prompt_dirty {
                                s.refresh_cache();
                                cx.emit(crate::store::StoreEvent::PromptCacheUpdated);
                            }
                        });
                        this.state.save_pending = true;
                    }

                    // Sync old state inputs (bridge)
                    this.sync_block_content(cx);

                    // Timers
                    if this.state.copy_feedback > 0 { this.state.copy_feedback = this.state.copy_feedback.saturating_sub(6); }
                    if this.state.save_status_timer > 0 {
                        this.state.save_status_timer = this.state.save_status_timer.saturating_sub(6);
                        if this.state.save_status_timer == 0 && this.state.save_status == "saved" {
                            this.state.save_status = "idle";
                            this.store.update(cx, |s, cx| {
                                s.save_status = "idle";
                                cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                            });
                        }
                    }

                    // Auto-save
                    if this.state.save_pending && this.state.save_timer == 0 {
                        this.state.save_timer = 5; // ~500ms
                    }
                    if this.state.save_timer > 0 {
                        this.state.save_timer -= 1;
                        if this.state.save_timer == 0 && this.state.save_pending {
                            this.state.save_status = "saved";
                            this.state.save_status_timer = 30;
                            this.state.save_pending = false;
                            this.save_to_backend();
                            this.store.update(cx, |s, cx| {
                                s.save_status = "saved";
                                cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                            });
                        }
                    }
                }).ok();
                if should_continue.is_none() { break; }
            }
        }).detach();
    }
}

impl Render for InkwellApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // PURE render — zero state mutation

        // One-time init
        if !self.state.inputs_initialized {
            self.ensure_block_inputs(window, cx);
            self.ensure_terminal_input(window, cx);
            self.state.inputs_initialized = true;
            Self::start_periodic_sync(cx);
        }

        match self.state.screen {
            Screen::Auth => self.render_auth(window, cx),
            Screen::Ide => self.render_ide(cx),
        }
    }
}
