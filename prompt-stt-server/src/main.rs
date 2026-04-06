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
use iced::{Element, Length, Task as IcedTask, Theme};
use models::ModelInfo;
use ollama::{OllamaState, OllamaStatus};
use server::ServerStatus;
use tokio::sync::watch;
use whisper_engine::WhisperEngine;

fn main() -> iced::Result {
    iced::application("Prompt AI Server", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window_size((560.0, 750.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    // Server
    ServerToggle(bool),
    // Whisper STT
    SelectModel(String),
    LoadModel,
    ModelLoaded(Result<(), String>),
    DownloadModel(String),
    DownloadDone(Result<(), String>),
    // Ollama
    OllamaUrlChanged(String),
    OllamaToggle(bool),
    OllamaRefreshed(OllamaStatus),
    RefreshOllama,
    // General
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

    // Download
    download_progress: Option<DownloadProgress>,
    download_progress_rx: watch::Receiver<Option<DownloadProgress>>,
    download_progress_tx: watch::Sender<Option<DownloadProgress>>,

    // Server status
    server_status_tx: watch::Sender<ServerStatus>,

    // Ollama
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
        let (status_tx, _status_rx) = watch::channel(ServerStatus {
            running: false,
            port: 8910,
            model_loaded: false,
            current_model: None,
            transcriptions_count: 0,
        });

        let installed = models::installed_models();
        let selected = installed.first().map(|m| m.id.clone());

        let ollama_state = OllamaState::new();

        let mut app = Self {
            engine,
            server_running: true,
            port: 8910,
            selected_model: selected,
            all_models,
            model_loaded: false,
            loading_model: false,
            load_error: None,
            download_progress: None,
            download_progress_rx: dl_rx,
            download_progress_tx: dl_tx,
            server_status_tx: status_tx,
            ollama_state: ollama_state.clone(),
            ollama_url: "http://localhost:11434".into(),
            ollama_enabled: true,
            ollama_connected: false,
            ollama_models: vec![],
            ollama_error: None,
            log_messages: vec!["Prompt AI Server demarre.".into()],
        };

        // Auto-start HTTP server
        {
            let port = app.port;
            let engine = app.engine.clone();
            let ollama = app.ollama_state.clone();
            let status_tx = app.server_status_tx.clone();
            app.log_messages.push(format!("Serveur demarre automatiquement sur le port {port}..."));
            app.log_messages.push("STT: /transcribe | LLM: /v1/chat/completions | API: /api/*".into());
            tokio::spawn(async move {
                let db = database::Database::open().expect("Failed to open database");
                server::start_server(port, engine, ollama, status_tx, db).await;
            });
        }

        // Initial Ollama check
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
                    self.log_messages
                        .push(format!("Serveur demarre sur le port {port}..."));
                    self.log_messages.push(format!(
                        "STT: /transcribe | LLM: /v1/chat/completions"
                    ));

                    tokio::spawn(async move {
                        let db = database::Database::open().expect("Failed to open database");
                        server::start_server(port, engine, ollama, status_tx, db).await;
                    });
                }
                IcedTask::none()
            }

            Message::SelectModel(id) => {
                self.selected_model = Some(id);
                IcedTask::none()
            }

            Message::LoadModel => {
                if let Some(ref id) = self.selected_model {
                    let model = self.all_models.iter().find(|m| m.id == *id).cloned();
                    if let Some(model) = model {
                        if models::is_model_installed(&model) {
                            self.loading_model = true;
                            self.load_error = None;
                            let engine = self.engine.clone();
                            let path = models::model_path(&model);
                            self.log_messages
                                .push(format!("Chargement {}...", model.name));

                            return IcedTask::perform(
                                async move {
                                    tokio::task::spawn_blocking(move || engine.load_model(&path))
                                        .await
                                        .unwrap_or_else(|e| Err(format!("Task error: {e}")))
                                },
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
                    Ok(()) => {
                        self.model_loaded = true;
                        self.load_error = None;
                        self.log_messages.push("Whisper charge!".into());
                    }
                    Err(e) => {
                        self.load_error = Some(e.clone());
                        self.log_messages.push(format!("Erreur: {e}"));
                    }
                }
                IcedTask::none()
            }

            Message::DownloadModel(id) => {
                let model = self.all_models.iter().find(|m| m.id == id).cloned();
                if let Some(model) = model {
                    let tx = self.download_progress_tx.clone();
                    self.log_messages
                        .push(format!("Telechargement {}...", model.name));
                    return IcedTask::perform(
                        async move { downloader::download_model(model, tx).await },
                        Message::DownloadDone,
                    );
                }
                IcedTask::none()
            }

            Message::DownloadDone(result) => {
                match &result {
                    Ok(()) => self.log_messages.push("Telechargement termine!".into()),
                    Err(e) => self.log_messages.push(format!("Erreur: {e}")),
                }
                self.download_progress = None;
                IcedTask::none()
            }

            // --- Ollama ---
            Message::OllamaUrlChanged(url) => {
                self.ollama_url = url.clone();
                let ollama = self.ollama_state.clone();
                tokio::spawn(async move {
                    let mut cfg = ollama.config.write().await;
                    cfg.url = url;
                });
                IcedTask::none()
            }

            Message::OllamaToggle(enabled) => {
                self.ollama_enabled = enabled;
                let ollama = self.ollama_state.clone();
                tokio::spawn(async move {
                    let mut cfg = ollama.config.write().await;
                    cfg.enabled = enabled;
                });
                if enabled {
                    return self.refresh_ollama();
                }
                IcedTask::none()
            }

            Message::RefreshOllama => self.refresh_ollama(),

            Message::OllamaRefreshed(status) => {
                self.ollama_connected = status.connected;
                self.ollama_models = status.models;
                self.ollama_error = status.error;
                if self.ollama_connected {
                    let count = self.ollama_models.len();
                    // Only log on first connection or change
                    let last = self.log_messages.last().cloned().unwrap_or_default();
                    if !last.contains("Ollama connecte") {
                        self.log_messages
                            .push(format!("Ollama connecte ({count} modeles)"));
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
        IcedTask::perform(
            async move {
                {
                    let mut cfg = ollama.config.write().await;
                    cfg.url = url;
                }
                ollama.refresh_status().await;
                ollama.status.read().await.clone()
            },
            Message::OllamaRefreshed,
        )
    }

    fn view(&self) -> Element<Message> {
        let installed: Vec<String> = self
            .all_models
            .iter()
            .filter(|m| models::is_model_installed(m))
            .map(|m| m.id.clone())
            .collect();

        let title = text!("Prompt AI Server").size(22);
        let subtitle = text!("STT (Whisper) + LLM (Ollama) pour Prompt IDE")
            .size(12)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.65));

        // --- Server ---
        let server_section = {
            let toggle = toggler(self.server_running)
                .label("Serveur HTTP")
                .on_toggle(Message::ServerToggle);

            let status_color = if self.server_running {
                iced::Color::from_rgb(0.2, 0.83, 0.6)
            } else {
                iced::Color::from_rgb(0.6, 0.6, 0.65)
            };

            let status_text = if self.server_running {
                format!("● En ligne — http://0.0.0.0:{}", self.port)
            } else {
                "○ Arrete".into()
            };

            column![
                text!("Serveur unifie").size(15),
                toggle,
                text!("{status_text}").size(12).color(status_color),
            ]
            .spacing(6)
        };

        // --- Ollama ---
        let ollama_section = {
            let toggle = toggler(self.ollama_enabled)
                .label("Ollama (LLM)")
                .on_toggle(Message::OllamaToggle);

            let url_input = text_input("http://localhost:11434", &self.ollama_url)
                .on_input(Message::OllamaUrlChanged)
                .size(12);

            let status_color = if self.ollama_connected {
                iced::Color::from_rgb(0.2, 0.83, 0.6)
            } else {
                iced::Color::from_rgb(0.97, 0.26, 0.26)
            };

            let status_text = if !self.ollama_enabled {
                "○ Desactive".to_string()
            } else if self.ollama_connected {
                format!("● Connecte — {} modele(s)", self.ollama_models.len())
            } else if let Some(ref e) = self.ollama_error {
                format!("✗ {e}")
            } else {
                "○ Deconnecte".to_string()
            };

            let refresh_btn =
                button(text!("Tester").size(11)).on_press(Message::RefreshOllama);

            let mut col = column![
                text!("Ollama (LLM proxy)").size(15),
                toggle,
                row![url_input, refresh_btn].spacing(6),
                text!("{status_text}").size(11).color(status_color),
            ]
            .spacing(6);

            // Show installed Ollama models
            if self.ollama_connected && !self.ollama_models.is_empty() {
                for m in &self.ollama_models {
                    let size_gb = m.size as f64 / 1_073_741_824.0;
                    let params = m
                        .parameter_size
                        .as_deref()
                        .unwrap_or("?");
                    let quant = m
                        .quantization_level
                        .as_deref()
                        .unwrap_or("");
                    let label = format!(
                        "  {} ({params}, {quant}, {:.1}Go)",
                        m.name, size_gb
                    );
                    col = col.push(
                        text!("{label}")
                            .size(11)
                            .color(iced::Color::from_rgb(0.55, 0.55, 0.6)),
                    );
                }
            }

            col
        };

        // --- Whisper STT ---
        let whisper_section = {
            let picker = pick_list(installed.clone(), self.selected_model.clone(), |id| {
                Message::SelectModel(id)
            })
            .placeholder("Selectionner...");

            let load_btn = if self.loading_model {
                button(text!("...").size(13))
            } else {
                button(text!("Charger").size(13)).on_press(Message::LoadModel)
            };

            let status = if self.model_loaded {
                text!("Pret")
                    .size(11)
                    .color(iced::Color::from_rgb(0.2, 0.83, 0.6))
            } else if let Some(ref e) = self.load_error {
                text!("Erreur: {e}")
                    .size(11)
                    .color(iced::Color::from_rgb(0.97, 0.26, 0.26))
            } else {
                text!("Aucun modele")
                    .size(11)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.65))
            };

            column![
                text!("Whisper (STT)").size(15),
                row![picker, load_btn].spacing(6),
                status,
            ]
            .spacing(6)
        };

        // --- Downloads ---
        let download_section = {
            let mut col = column![text!("Modeles Whisper").size(14)].spacing(4);

            for model in &self.all_models {
                let is_installed = models::is_model_installed(model);
                let label = format!("{} ({}Mo)", model.name, model.size_mb);

                if is_installed {
                    col = col.push(
                        row![
                            text!("{label}").size(11),
                            Space::with_width(Length::Fill),
                            text!("✓")
                                .size(11)
                                .color(iced::Color::from_rgb(0.2, 0.83, 0.6)),
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                    );
                } else {
                    let dl_btn = button(text!("DL").size(10))
                        .on_press(Message::DownloadModel(model.id.clone()));
                    col = col.push(
                        row![
                            text!("{label}").size(11),
                            Space::with_width(Length::Fill),
                            dl_btn,
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                    );
                }
            }

            if let Some(ref prog) = self.download_progress {
                let pct = if prog.total_bytes > 0 {
                    prog.downloaded_bytes as f32 / prog.total_bytes as f32
                } else {
                    0.0
                };
                let mb_done = prog.downloaded_bytes / (1024 * 1024);
                let mb_total = prog.total_bytes / (1024 * 1024);
                col = col
                    .push(progress_bar(0.0..=1.0, pct).height(5))
                    .push(
                        text!("{mb_done}/{mb_total} Mo")
                            .size(10)
                            .color(iced::Color::from_rgb(0.6, 0.6, 0.65)),
                    );
            }

            col
        };

        // --- Log ---
        let log_section = {
            let mut col = column![text!("Journal").size(14)].spacing(2);
            for msg in self.log_messages.iter().rev().take(6) {
                col = col.push(
                    text!("{msg}")
                        .size(10)
                        .color(iced::Color::from_rgb(0.45, 0.45, 0.5)),
                );
            }
            col
        };

        let content = column![
            title,
            subtitle,
            horizontal_rule(1),
            server_section,
            horizontal_rule(1),
            ollama_section,
            horizontal_rule(1),
            whisper_section,
            horizontal_rule(1),
            download_section,
            horizontal_rule(1),
            log_section,
        ]
        .spacing(12)
        .padding(20)
        .width(Length::Fill);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
