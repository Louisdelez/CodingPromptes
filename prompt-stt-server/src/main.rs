mod api_routes;
mod database;
mod downloader;
mod jwt_auth;
mod models;
mod ollama;
mod server;
mod whisper_engine;

use downloader::DownloadProgress;
use iced::widget::{
    button, column, container, horizontal_rule, pick_list, progress_bar, row, scrollable, text,
    text_input, toggler, Space,
};
use iced::{border, Color, Element, Length, Task as IcedTask, Theme};
use models::ModelInfo;
use ollama::{OllamaState, OllamaStatus};
use server::ServerStatus;
use tokio::sync::watch;
use whisper_engine::WhisperEngine;

// Colors
const ACCENT: Color = Color::from_rgb(0.39, 0.40, 0.95);
const SUCCESS: Color = Color::from_rgb(0.20, 0.83, 0.60);
const DANGER: Color = Color::from_rgb(0.97, 0.26, 0.26);
const WARNING: Color = Color::from_rgb(0.98, 0.75, 0.14);
const MUTED: Color = Color::from_rgb(0.45, 0.45, 0.50);
const SUBTLE: Color = Color::from_rgb(0.55, 0.55, 0.60);
const CARD_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.03);
const CARD_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(CARD_BG)),
        border: border::rounded(10).color(CARD_BORDER).width(1),
        ..Default::default()
    }
}

fn main() -> iced::Result {
    iced::application("Inkwell GPU Server", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window_size((600.0, 800.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    ServerToggle(bool),
    SelectModel(String),
    LoadModel,
    ModelLoaded(Result<(), String>),
    DownloadModel(String),
    DownloadDone(Result<(), String>),
    OllamaUrlChanged(String),
    OllamaToggle(bool),
    OllamaRefreshed(OllamaStatus),
    RefreshOllama,
    Tick,
}

struct App {
    engine: WhisperEngine,
    server_running: bool,
    port: u16,
    selected_model: Option<String>,
    all_models: Vec<ModelInfo>,
    model_loaded: bool,
    loading_model: bool,
    load_error: Option<String>,
    download_progress: Option<DownloadProgress>,
    download_progress_rx: watch::Receiver<Option<DownloadProgress>>,
    download_progress_tx: watch::Sender<Option<DownloadProgress>>,
    #[allow(dead_code)]
    server_status_tx: watch::Sender<ServerStatus>,
    ollama_state: OllamaState,
    ollama_url: String,
    ollama_enabled: bool,
    ollama_connected: bool,
    ollama_models: Vec<ollama::OllamaModel>,
    ollama_error: Option<String>,
    log_messages: Vec<String>,
}

impl App {
    fn new() -> (Self, IcedTask<Message>) {
        let engine = WhisperEngine::new();
        let all_models = models::available_models();
        let (dl_tx, dl_rx) = watch::channel(None);
        let (status_tx, _) = watch::channel(ServerStatus {
            running: false, port: 8910, model_loaded: false,
            current_model: None, transcriptions_count: 0,
        });
        let installed = models::installed_models();
        let selected = installed.first().map(|m| m.id.clone());
        let ollama_state = OllamaState::new();

        let mut app = Self {
            engine, server_running: true, port: 8910,
            selected_model: selected, all_models,
            model_loaded: false, loading_model: false, load_error: None,
            download_progress: None, download_progress_rx: dl_rx, download_progress_tx: dl_tx,
            server_status_tx: status_tx,
            ollama_state: ollama_state.clone(),
            ollama_url: "http://localhost:11434".into(),
            ollama_enabled: true, ollama_connected: false,
            ollama_models: vec![], ollama_error: None,
            log_messages: vec!["Inkwell GPU Server started".into()],
        };

        // Auto-start HTTP server
        {
            let port = app.port;
            let engine = app.engine.clone();
            let ollama = app.ollama_state.clone();
            let status_tx = app.server_status_tx.clone();
            app.log_messages.push(format!("Listening on port {port}"));
            tokio::spawn(async move {
                let db = database::Database::open().expect("Failed to open database");
                server::start_server(port, engine, ollama, status_tx, db).await;
            });
        }

        let ollama_check = ollama_state.clone();
        let init_task = IcedTask::perform(
            async move {
                ollama_check.refresh_status().await;
                ollama_check.status.read().await.clone()
            },
            Message::OllamaRefreshed,
        );

        (app, init_task)
    }

    fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            Message::ServerToggle(on) => {
                if on && !self.server_running {
                    let port = self.port;
                    let engine = self.engine.clone();
                    let ollama = self.ollama_state.clone();
                    let status_tx = self.server_status_tx.clone();
                    self.server_running = true;
                    self.log_messages.push(format!("Server started on port {port}"));
                    tokio::spawn(async move {
                        let db = database::Database::open().expect("Failed to open database");
                        server::start_server(port, engine, ollama, status_tx, db).await;
                    });
                }
                IcedTask::none()
            }
            Message::SelectModel(id) => { self.selected_model = Some(id); IcedTask::none() }
            Message::LoadModel => {
                if let Some(ref id) = self.selected_model {
                    if let Some(model) = self.all_models.iter().find(|m| m.id == *id).cloned() {
                        if models::is_model_installed(&model) {
                            self.loading_model = true;
                            self.load_error = None;
                            let engine = self.engine.clone();
                            let path = models::model_path(&model);
                            self.log_messages.push(format!("Loading {}...", model.name));
                            return IcedTask::perform(
                                async move { tokio::task::spawn_blocking(move || engine.load_model(&path)).await.unwrap_or_else(|e| Err(format!("{e}"))) },
                                Message::ModelLoaded,
                            );
                        }
                    }
                }
                IcedTask::none()
            }
            Message::ModelLoaded(result) => {
                self.loading_model = false;
                match result {
                    Ok(()) => { self.model_loaded = true; self.load_error = None; self.log_messages.push("Whisper model loaded".into()); }
                    Err(e) => { self.load_error = Some(e.clone()); self.log_messages.push(format!("Error: {e}")); }
                }
                IcedTask::none()
            }
            Message::DownloadModel(id) => {
                if let Some(model) = self.all_models.iter().find(|m| m.id == id).cloned() {
                    let tx = self.download_progress_tx.clone();
                    self.log_messages.push(format!("Downloading {}...", model.name));
                    return IcedTask::perform(async move { downloader::download_model(model, tx).await }, Message::DownloadDone);
                }
                IcedTask::none()
            }
            Message::DownloadDone(result) => {
                match &result {
                    Ok(()) => self.log_messages.push("Download complete".into()),
                    Err(e) => self.log_messages.push(format!("Error: {e}")),
                }
                self.download_progress = None;
                IcedTask::none()
            }
            Message::OllamaUrlChanged(url) => {
                self.ollama_url = url.clone();
                let ollama = self.ollama_state.clone();
                tokio::spawn(async move { ollama.config.write().await.url = url; });
                IcedTask::none()
            }
            Message::OllamaToggle(enabled) => {
                self.ollama_enabled = enabled;
                let ollama = self.ollama_state.clone();
                tokio::spawn(async move { ollama.config.write().await.enabled = enabled; });
                if enabled { return self.refresh_ollama(); }
                IcedTask::none()
            }
            Message::RefreshOllama => self.refresh_ollama(),
            Message::OllamaRefreshed(status) => {
                self.ollama_connected = status.connected;
                self.ollama_models = status.models;
                self.ollama_error = status.error;
                if self.ollama_connected {
                    let count = self.ollama_models.len();
                    let last = self.log_messages.last().cloned().unwrap_or_default();
                    if !last.contains("Ollama connected") {
                        self.log_messages.push(format!("Ollama connected ({count} models)"));
                    }
                }
                IcedTask::none()
            }
            Message::Tick => {
                if self.download_progress_rx.has_changed().unwrap_or(false) {
                    self.download_progress = self.download_progress_rx.borrow_and_update().clone();
                }
                IcedTask::none()
            }
        }
    }

    fn refresh_ollama(&self) -> IcedTask<Message> {
        let ollama = self.ollama_state.clone();
        let url = self.ollama_url.clone();
        IcedTask::perform(async move {
            { ollama.config.write().await.url = url; }
            ollama.refresh_status().await;
            ollama.status.read().await.clone()
        }, Message::OllamaRefreshed)
    }

    fn view(&self) -> Element<'_, Message> {
        let installed: Vec<String> = self.all_models.iter()
            .filter(|m| models::is_model_installed(m))
            .map(|m| m.id.clone()).collect();

        // === Header ===
        let header = container(
            row![
                column![
                    text!("Inkwell").size(24).color(Color::WHITE),
                    text!("GPU Server").size(12).color(ACCENT),
                ].spacing(2),
                Space::with_width(Length::Fill),
                // Status badge
                container(
                    row![
                        text!("*").size(10).color(if self.server_running { SUCCESS } else { DANGER }),
                        text(if self.server_running { "Online" } else { "Offline" }).size(11).color(Color::WHITE),
                    ].spacing(4).align_y(iced::Alignment::Center)
                ).padding(6)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                    border: border::rounded(20),
                    ..Default::default()
                }),
            ].align_y(iced::Alignment::Center)
        ).padding(18);

        // === Server Card ===
        let server_card = container(
            column![
                row![
                    text!("S").size(14).color(ACCENT),
                    text!("Server").size(14).color(Color::WHITE),
                    Space::with_width(Length::Fill),
                    toggler(self.server_running).on_toggle(Message::ServerToggle),
                ].spacing(8).align_y(iced::Alignment::Center),
                text(format!("http://0.0.0.0:{}", self.port)).size(11).color(ACCENT),
                row![
                    text!("STT").size(9).color(MUTED),
                    text!("|").size(9).color(MUTED),
                    text!("LLM Proxy").size(9).color(MUTED),
                    text!("|").size(9).color(MUTED),
                    text!("API").size(9).color(MUTED),
                ].spacing(4),
            ].spacing(6).padding(14)
        ).style(card_style);

        // === Ollama Card ===
        let ollama_status_color = if self.ollama_connected { SUCCESS } else if self.ollama_enabled { DANGER } else { MUTED };
        let ollama_status_text = if !self.ollama_enabled { "Disabled".to_string() }
            else if self.ollama_connected { format!("Connected — {} model(s)", self.ollama_models.len()) }
            else if let Some(ref e) = self.ollama_error { format!("Error: {}", &e[..e.len().min(40)]) }
            else { "Disconnected".to_string() };

        let mut ollama_card_content = column![
            row![
                text!("O").size(14).color(ACCENT),
                text!("Ollama (LLM)").size(14).color(Color::WHITE),
                Space::with_width(Length::Fill),
                toggler(self.ollama_enabled).on_toggle(Message::OllamaToggle),
            ].spacing(8).align_y(iced::Alignment::Center),
            row![
                text!("*").size(8).color(ollama_status_color),
                text!("{ollama_status_text}").size(11).color(ollama_status_color),
            ].spacing(4).align_y(iced::Alignment::Center),
            row![
                text_input("http://localhost:11434", &self.ollama_url)
                    .on_input(Message::OllamaUrlChanged).size(11),
                button(text!("Test").size(10)).on_press(Message::RefreshOllama).style(button::secondary),
            ].spacing(6),
        ].spacing(8);

        // Ollama models list
        if self.ollama_connected && !self.ollama_models.is_empty() {
            let mut models_col = column![].spacing(3);
            for m in &self.ollama_models {
                let size_gb = m.size as f64 / 1_073_741_824.0;
                let params = m.parameter_size.as_deref().unwrap_or("?");
                let quant = m.quantization_level.as_deref().unwrap_or("");
                models_col = models_col.push(
                    row![
                        text!("  >").size(8).color(ACCENT),
                        text!("{}", m.name).size(10).color(Color::WHITE),
                        Space::with_width(Length::Fill),
                        text!("{params}").size(9).color(MUTED),
                        text!("{quant}").size(9).color(MUTED),
                        text!("{:.1}G", size_gb).size(9).color(SUBTLE),
                    ].spacing(6).align_y(iced::Alignment::Center)
                );
            }
            ollama_card_content = ollama_card_content.push(models_col);
        }

        let ollama_card = container(ollama_card_content.padding(14)).style(card_style);

        // === Whisper Card ===
        let whisper_status = if self.model_loaded { ("*", "Ready", SUCCESS) }
            else if self.loading_model { ("-", "Loading...", WARNING) }
            else if let Some(ref e) = self.load_error { ("x", e.as_str(), DANGER) }
            else { ("-", "No model loaded", MUTED) };

        let whisper_card = container(
            column![
                row![
                    text!("W").size(14).color(ACCENT),
                    text!("Whisper (STT)").size(14).color(Color::WHITE),
                    Space::with_width(Length::Fill),
                    text(whisper_status.0).size(10).color(whisper_status.2),
                    text(whisper_status.1).size(10).color(whisper_status.2),
                ].spacing(6).align_y(iced::Alignment::Center),
                row![
                    pick_list(installed, self.selected_model.clone(), |id| Message::SelectModel(id))
                        .placeholder("Select model..."),
                    if self.loading_model {
                        button(text!("...").size(12))
                    } else {
                        button(text!("Load").size(12)).on_press(Message::LoadModel).style(button::primary)
                    },
                ].spacing(6),
            ].spacing(8).padding(14)
        ).style(card_style);

        // === Downloads Card ===
        let mut dl_content = column![
            row![
                text!("D").size(14).color(ACCENT),
                text!("Whisper Models").size(14).color(Color::WHITE),
            ].spacing(8),
        ].spacing(6);

        for model in &self.all_models {
            let is_installed = models::is_model_installed(model);
            let name = model.name.clone();
            let size = format!("{}MB", model.size_mb);

            dl_content = dl_content.push(
                row![
                    if is_installed {
                        text!("ok").size(11).color(SUCCESS)
                    } else {
                        text!("-").size(11).color(MUTED)
                    },
                    text!("{name}").size(11).color(if is_installed { Color::WHITE } else { SUBTLE }),
                    Space::with_width(Length::Fill),
                    text!("{size}").size(10).color(MUTED),
                    if !is_installed {
                        button(text!("Download").size(9)).on_press(Message::DownloadModel(model.id.clone())).style(button::secondary)
                    } else {
                        button(text!("Installed").size(9)).style(button::text)
                    },
                ].spacing(8).align_y(iced::Alignment::Center)
            );
        }

        if let Some(ref prog) = self.download_progress {
            let pct = if prog.total_bytes > 0 { prog.downloaded_bytes as f32 / prog.total_bytes as f32 } else { 0.0 };
            let mb_done = prog.downloaded_bytes / (1024 * 1024);
            let mb_total = prog.total_bytes / (1024 * 1024);
            dl_content = dl_content
                .push(progress_bar(0.0..=1.0, pct).height(4))
                .push(text!("{mb_done}/{mb_total} MB").size(10).color(MUTED));
        }

        let download_card = container(dl_content.padding(14)).style(card_style);

        // === Log Card ===
        let mut log_content = column![
            text!("Activity").size(12).color(SUBTLE),
        ].spacing(3);
        for msg in self.log_messages.iter().rev().take(5) {
            log_content = log_content.push(
                text!("- {msg}").size(10).color(MUTED)
            );
        }
        let log_card = container(log_content.padding(12)).style(card_style);

        // === Layout ===
        let content = column![
            header,
            scrollable(
                column![
                    server_card,
                    ollama_card,
                    whisper_card,
                    download_card,
                    log_card,
                ].spacing(10).padding(20)
            ).height(Length::Fill),
        ].spacing(4);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
