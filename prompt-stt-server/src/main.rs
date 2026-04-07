mod api_routes;
mod database;
mod downloader;
mod fleet;
mod hardware;
mod hw_widgets;
mod i18n;
mod jwt_auth;
mod models;
mod ollama;
mod server;
mod terminal;
mod whisper_engine;

use downloader::DownloadProgress;
use i18n::{Lang, T};
use iced::widget::{
    button, column, container, horizontal_rule, image, pick_list, progress_bar, row, scrollable,
    text, text_input, toggler, Space,
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
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window_size((600.0, 800.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    ServerToggle(bool),
    SelectModel(String),
    WhisperUseGpu(bool),
    LoadModel,
    ModelLoaded(Result<(), String>),
    DownloadModel(String),
    DownloadDone(Result<(), String>),
    OllamaUrlChanged(String),
    OllamaToggle(bool),
    OllamaRefreshed(OllamaStatus),
    RefreshOllama,
    LangChanged(Lang),
    // Hardware
    RefreshHardware,
    // Fleet
    FleetApiUrlChanged(String),
    FleetEmailChanged(String),
    FleetPasswordChanged(String),
    FleetNodeNameChanged(String),
    FleetConnect,
    FleetConnectResult(Result<String, String>),
    FleetDisconnect,
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
    whisper_use_gpu: bool,
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
    lang: Lang,
    hw: hardware::HardwareInfo,
    cpu_history: std::collections::VecDeque<f32>,
    ram_history: std::collections::VecDeque<f32>,
    gpu_history: std::collections::VecDeque<f32>,
    vram_history: std::collections::VecDeque<f32>,
    // Fleet
    fleet: fleet::FleetState,
    fleet_api_url: String,
    fleet_email: String,
    fleet_password: String,
    fleet_node_name: String,
    fleet_connected: bool,
    fleet_connecting: bool,
    fleet_error: Option<String>,
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

        let lang = i18n::detect_system_lang();
        let hw = hardware::HardwareInfo::detect();
        let fleet = fleet::FleetState::new();
        let fleet_config_snapshot = {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { fleet.config.read().await.clone() })
        };
        let fleet_connected = !fleet_config_snapshot.jwt_token.is_empty();

        let mut app = Self {
            engine, server_running: true, port: 8910,
            selected_model: selected, all_models,
            model_loaded: false, loading_model: false, whisper_use_gpu: false, load_error: None,
            download_progress: None, download_progress_rx: dl_rx, download_progress_tx: dl_tx,
            server_status_tx: status_tx,
            ollama_state: ollama_state.clone(),
            ollama_url: "http://localhost:11434".into(),
            ollama_enabled: true, ollama_connected: false,
            ollama_models: vec![], ollama_error: None,
            log_messages: vec![T::server_started(lang).into()],
            lang,
            hw,
            cpu_history: std::collections::VecDeque::with_capacity(60),
            ram_history: std::collections::VecDeque::with_capacity(60),
            gpu_history: std::collections::VecDeque::with_capacity(60),
            vram_history: std::collections::VecDeque::with_capacity(60),
            fleet: fleet.clone(),
            fleet_api_url: fleet_config_snapshot.api_url.clone(),
            fleet_email: fleet_config_snapshot.user_email.clone(),
            fleet_password: String::new(),
            fleet_node_name: if fleet_config_snapshot.node_name.is_empty() {
                hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "GPU Node".into())
            } else {
                fleet_config_snapshot.node_name.clone()
            },
            fleet_connected,
            fleet_connecting: false,
            fleet_error: None,
        };

        // Auto-start HTTP server
        {
            let port = app.port;
            let engine = app.engine.clone();
            let ollama = app.ollama_state.clone();
            let status_tx = app.server_status_tx.clone();
            app.log_messages.push(T::listening_on(app.lang, port));
            tokio::spawn(async move {
                let db = database::Database::open().expect("Failed to open database");
                server::start_server(port, engine, ollama, status_tx, db).await;
            });
        }

        // Auto-start fleet heartbeat if configured
        if fleet_connected {
            fleet.start_heartbeat_loop(app.engine.clone(), ollama_state.clone(), app.port);
            app.log_messages.push("Fleet connected".into());
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
                    self.log_messages.push(T::server_started_on(self.lang, port));
                    tokio::spawn(async move {
                        let db = database::Database::open().expect("Failed to open database");
                        server::start_server(port, engine, ollama, status_tx, db).await;
                    });
                }
                IcedTask::none()
            }
            Message::SelectModel(id) => { self.selected_model = Some(id); IcedTask::none() }
            Message::RefreshHardware => { self.hw = hardware::HardwareInfo::detect(); IcedTask::none() }
            Message::WhisperUseGpu(v) => { self.whisper_use_gpu = v; IcedTask::none() }
            Message::LoadModel => {
                if let Some(ref id) = self.selected_model {
                    if let Some(model) = self.all_models.iter().find(|m| m.id == *id).cloned() {
                        if models::is_model_installed(&model) {
                            self.loading_model = true;
                            self.load_error = None;
                            let engine = self.engine.clone();
                            let path = models::model_path(&model);
                            let use_gpu = self.whisper_use_gpu;
                            let device_label = if use_gpu { "GPU" } else { "CPU" };
                            self.log_messages.push(format!("{} ({})", T::loading_model(self.lang, &model.name), device_label));
                            return IcedTask::perform(
                                async move { tokio::task::spawn_blocking(move || engine.load_model(&path, use_gpu)).await.unwrap_or_else(|e| Err(format!("{e}"))) },
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
                    Ok(()) => { self.model_loaded = true; self.load_error = None; self.log_messages.push(T::model_loaded(self.lang).into()); }
                    Err(e) => { self.load_error = Some(e.clone()); self.log_messages.push(format!("Error: {e}")); }
                }
                IcedTask::none()
            }
            Message::DownloadModel(id) => {
                if let Some(model) = self.all_models.iter().find(|m| m.id == id).cloned() {
                    let tx = self.download_progress_tx.clone();
                    self.log_messages.push(T::downloading_model(self.lang, &model.name));
                    return IcedTask::perform(async move { downloader::download_model(model, tx).await }, Message::DownloadDone);
                }
                IcedTask::none()
            }
            Message::DownloadDone(result) => {
                match &result {
                    Ok(()) => self.log_messages.push(T::download_complete(self.lang).into()),
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
            Message::LangChanged(lang) => { self.lang = lang; IcedTask::none() }
            Message::FleetApiUrlChanged(url) => { self.fleet_api_url = url; IcedTask::none() }
            Message::FleetEmailChanged(email) => { self.fleet_email = email; IcedTask::none() }
            Message::FleetPasswordChanged(pw) => { self.fleet_password = pw; IcedTask::none() }
            Message::FleetNodeNameChanged(name) => { self.fleet_node_name = name; IcedTask::none() }
            Message::FleetConnect => {
                self.fleet_connecting = true;
                self.fleet_error = None;
                let fleet = self.fleet.clone();
                let api_url = self.fleet_api_url.clone();
                let email = self.fleet_email.clone();
                let password = self.fleet_password.clone();
                let node_name = self.fleet_node_name.clone();
                let port = self.port;
                let engine = self.engine.clone();
                let ollama = self.ollama_state.clone();
                IcedTask::perform(async move {
                    let result = fleet.login(&api_url, &email, &password, &node_name, port).await;
                    if result.is_ok() {
                        fleet.start_heartbeat_loop(engine, ollama, port);
                    }
                    result
                }, Message::FleetConnectResult)
            }
            Message::FleetConnectResult(result) => {
                self.fleet_connecting = false;
                match result {
                    Ok(_id) => {
                        self.fleet_connected = true;
                        self.fleet_error = None;
                        self.fleet_password.clear();
                        self.log_messages.push("Fleet: connected".into());
                    }
                    Err(e) => {
                        self.fleet_error = Some(e.clone());
                        self.log_messages.push(format!("Fleet: {e}"));
                    }
                }
                IcedTask::none()
            }
            Message::FleetDisconnect => {
                let fleet = self.fleet.clone();
                tokio::spawn(async move { fleet.disconnect().await; });
                self.fleet_connected = false;
                self.log_messages.push("Fleet: disconnected".into());
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
                    if !last.contains("Ollama") {
                        self.log_messages.push(T::ollama_connected(self.lang, count));
                    }
                }
                IcedTask::none()
            }
            Message::Tick => {
                if self.download_progress_rx.has_changed().unwrap_or(false) {
                    self.download_progress = self.download_progress_rx.borrow_and_update().clone();
                }
                self.hw.refresh();
                // Push history (keep last 60 points)
                if self.cpu_history.len() >= 60 { self.cpu_history.pop_front(); }
                self.cpu_history.push_back(self.hw.cpu.usage_percent);
                if self.ram_history.len() >= 60 { self.ram_history.pop_front(); }
                self.ram_history.push_back(self.hw.ram.usage_percent);
                if let Some(ref gpu) = self.hw.gpu {
                    if self.gpu_history.len() >= 60 { self.gpu_history.pop_front(); }
                    self.gpu_history.push_back(gpu.gpu_utilization as f32);
                    if self.vram_history.len() >= 60 { self.vram_history.pop_front(); }
                    self.vram_history.push_back(gpu.vram_usage_percent);
                }
                IcedTask::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(2)).map(|_| Message::Tick)
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
        let l = self.lang;
        let installed: Vec<String> = self.all_models.iter()
            .filter(|m| models::is_model_installed(m))
            .map(|m| m.id.clone()).collect();

        // === Header ===
        let logo_handle = image::Handle::from_bytes(include_bytes!("../assets/logo-96.png").to_vec());
        let logo = image(logo_handle).width(64).height(64);
        let header = container(
            row![
                logo,
                column![
                    text!("Inkwell").size(24).color(Color::WHITE),
                    text(T::gpu_server(l)).size(12).color(ACCENT),
                ].spacing(2),
                Space::with_width(Length::Fill),
                // Lang selector
                pick_list(Lang::ALL.as_slice(), Some(self.lang), Message::LangChanged),
                // Status badge
                container(
                    row![
                        text!("*").size(10).color(if self.server_running { SUCCESS } else { DANGER }),
                        text(if self.server_running { T::online(l) } else { T::offline(l) }).size(11).color(Color::WHITE),
                    ].spacing(4).align_y(iced::Alignment::Center)
                ).padding(6)
                .style(|_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                    border: border::rounded(20),
                    ..Default::default()
                }),
            ].spacing(8).align_y(iced::Alignment::Center)
        ).padding(18);

        // === Server Card ===
        let server_card = container(
            column![
                row![
                    text!("S").size(14).color(ACCENT),
                    text(T::server(l)).size(14).color(Color::WHITE),
                    Space::with_width(Length::Fill),
                    toggler(self.server_running).on_toggle(Message::ServerToggle),
                ].spacing(8).align_y(iced::Alignment::Center),
                text(format!("http://0.0.0.0:{}", self.port)).size(11).color(ACCENT),
                row![
                    text!("STT").size(9).color(MUTED),
                    text!("|").size(9).color(MUTED),
                    text(T::proxy_llm(l)).size(9).color(MUTED),
                    text!("|").size(9).color(MUTED),
                    text!("API").size(9).color(MUTED),
                ].spacing(4),
            ].spacing(6).padding(14)
        ).style(card_style);

        // === Hardware Card ===
        let cpu_usage = self.hw.cpu.usage_percent;
        let ram_usage = self.hw.ram.usage_percent;
        let cpu_dyn_color = hw_widgets::usage_color(cpu_usage, hw_widgets::CPU_COLOR);
        let ram_dyn_color = hw_widgets::usage_color(ram_usage, hw_widgets::RAM_COLOR);

        let mut hw_content = column![
            // Header
            row![
                text!("H").size(14).color(ACCENT),
                text!("Hardware").size(14).color(Color::WHITE),
                Space::with_width(Length::Fill),
                text!("{}", self.hw.os).size(9).color(MUTED),
            ].spacing(6).align_y(iced::Alignment::Center),

            // CPU
            row![
                hw_widgets::ring_gauge(cpu_usage, cpu_dyn_color, "CPU", 70.0),
                column![
                    row![
                        text!("CPU").size(12).color(hw_widgets::CPU_COLOR),
                        Space::with_width(Length::Fill),
                        text!("{:.0}%", cpu_usage).size(16).color(cpu_dyn_color),
                    ].align_y(iced::Alignment::Center),
                    hw_widgets::sparkline(&self.cpu_history, hw_widgets::CPU_COLOR, 60, 40.0),
                    text!("{}", self.hw.cpu.name).size(9).color(SUBTLE),
                    text!("{} cores / {} threads", self.hw.cpu.cores, self.hw.cpu.threads).size(9).color(MUTED),
                ].spacing(4).width(Length::Fill),
            ].spacing(12).align_y(iced::Alignment::Center),

            horizontal_rule(1),

            // RAM
            row![
                hw_widgets::ring_gauge(ram_usage, ram_dyn_color, "RAM", 70.0),
                column![
                    row![
                        text!("RAM").size(12).color(hw_widgets::RAM_COLOR),
                        Space::with_width(Length::Fill),
                        text!("{:.1} / {:.1} Go", self.hw.ram.used_gb, self.hw.ram.total_gb).size(11).color(Color::WHITE),
                        text!("{:.0}%", ram_usage).size(16).color(ram_dyn_color),
                    ].spacing(6).align_y(iced::Alignment::Center),
                    hw_widgets::sparkline(&self.ram_history, hw_widgets::RAM_COLOR, 60, 40.0),
                    row![
                        text!("Disponible:").size(9).color(MUTED),
                        text!("{:.1} Go", self.hw.ram.available_gb).size(9).color(SUCCESS),
                    ].spacing(4),
                ].spacing(4).width(Length::Fill),
            ].spacing(12).align_y(iced::Alignment::Center),
        ].spacing(14);

        // GPU section
        if let Some(ref gpu) = self.hw.gpu {
            let gpu_load = gpu.gpu_utilization as f32;
            let vram_pct = gpu.vram_usage_percent;
            let gpu_dyn_color = hw_widgets::usage_color(gpu_load, hw_widgets::GPU_COLOR);
            let vram_dyn_color = hw_widgets::usage_color(vram_pct, hw_widgets::VRAM_COLOR);
            let temp_color = if gpu.temperature > 80 { DANGER } else if gpu.temperature > 60 { WARNING } else { hw_widgets::GPU_COLOR };

            hw_content = hw_content
                .push(horizontal_rule(1))
                .push(
                    row![
                        hw_widgets::ring_gauge(gpu_load, gpu_dyn_color, "GPU", 70.0),
                        column![
                            row![
                                text!("GPU").size(12).color(hw_widgets::GPU_COLOR),
                                Space::with_width(Length::Fill),
                                text!("{}C", gpu.temperature).size(11).color(temp_color),
                                text!("{:.0}%", gpu_load).size(16).color(gpu_dyn_color),
                            ].spacing(6).align_y(iced::Alignment::Center),
                            hw_widgets::sparkline(&self.gpu_history, hw_widgets::GPU_COLOR, 60, 40.0),
                            text!("{}", gpu.name).size(9).color(SUBTLE),
                        ].spacing(4).width(Length::Fill),
                    ].spacing(12).align_y(iced::Alignment::Center)
                )
                .push(horizontal_rule(1))
                .push(
                    row![
                        hw_widgets::ring_gauge(vram_pct, vram_dyn_color, "VRAM", 70.0),
                        column![
                            row![
                                text!("VRAM").size(12).color(hw_widgets::VRAM_COLOR),
                                Space::with_width(Length::Fill),
                                text!("{:.1} / {:.1} Go", gpu.vram_used_mb as f64 / 1024.0, gpu.vram_total_mb as f64 / 1024.0).size(11).color(Color::WHITE),
                                text!("{:.0}%", vram_pct).size(16).color(vram_dyn_color),
                            ].spacing(6).align_y(iced::Alignment::Center),
                            hw_widgets::sparkline(&self.vram_history, hw_widgets::VRAM_COLOR, 60, 40.0),
                            row![
                                text!("Disponible:").size(9).color(MUTED),
                                text!("{:.1} Go", gpu.vram_free_mb as f64 / 1024.0).size(9).color(SUCCESS),
                                text!("|").size(9).color(CARD_BORDER),
                                text!("Driver {}", gpu.driver_version).size(9).color(MUTED),
                                text!("|").size(9).color(CARD_BORDER),
                                text!("CUDA {}", gpu.cuda_version).size(9).color(MUTED),
                            ].spacing(4),
                        ].spacing(4).width(Length::Fill),
                    ].spacing(12).align_y(iced::Alignment::Center)
                );
        } else {
            hw_content = hw_content
                .push(horizontal_rule(1))
                .push(
                    column![
                        row![
                            text!("GPU").size(12).color(DANGER),
                            Space::with_width(Length::Fill),
                            text!("Non detecte").size(10).color(DANGER),
                        ].align_y(iced::Alignment::Center),
                        text!("nvidia-smi introuvable — CPU uniquement").size(9).color(MUTED),
                    ].spacing(4)
                );
        }

        let hardware_card = container(hw_content.padding(16)).style(card_style);

        // === Ollama Card ===
        let ollama_status_color = if self.ollama_connected { SUCCESS } else if self.ollama_enabled { DANGER } else { MUTED };
        let ollama_status_text = if !self.ollama_enabled { T::disabled(l).to_string() }
            else if self.ollama_connected { T::connected_models(l, self.ollama_models.len()) }
            else if let Some(ref e) = self.ollama_error { format!("Error: {}", &e[..e.len().min(40)]) }
            else { T::disconnected(l).to_string() };

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
                button(text(T::test(l)).size(10)).on_press(Message::RefreshOllama).style(button::secondary),
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
        let whisper_status = if self.model_loaded { ("*", T::ready(l), SUCCESS) }
            else if self.loading_model { ("-", T::loading(l), WARNING) }
            else if self.load_error.is_some() { ("x", T::no_model_loaded(l), DANGER) }
            else { ("-", T::no_model_loaded(l), MUTED) };

        let gpu_available = self.hw.has_gpu();

        // Find hardware reqs for selected model
        let model_reqs = hardware::whisper_model_reqs();
        let selected_reqs = self.selected_model.as_ref().and_then(|sel| {
            let model = self.all_models.iter().find(|m| m.id == *sel)?;
            model_reqs.iter().find(|r| r.model_name == model.name).cloned()
        });

        let cpu_ram_free = self.hw.ram.available_gb;
        let gpu_vram_free = self.hw.gpu.as_ref().map(|g| g.vram_free_mb as f64 / 1024.0).unwrap_or(0.0);

        let mut whisper_content = column![
            // Header
            row![
                text!("W").size(14).color(ACCENT),
                text!("Whisper (STT)").size(14).color(Color::WHITE),
                Space::with_width(Length::Fill),
                text(whisper_status.0).size(10).color(whisper_status.2),
                text(whisper_status.1).size(10).color(whisper_status.2),
            ].spacing(6).align_y(iced::Alignment::Center),

            // Model selector
            pick_list(installed, self.selected_model.clone(), |id| Message::SelectModel(id))
                .placeholder(T::select_model(l)).width(Length::Fill),
        ].spacing(10);

        // Device selection (only show when a model is selected)
        if let Some(ref reqs) = selected_reqs {
            let can_cpu = cpu_ram_free >= reqs.cpu_ram_gb * 1.2;
            let can_gpu = gpu_available && gpu_vram_free >= reqs.gpu_vram_gb * 0.9;

            // CPU button
            let cpu_selected = !self.whisper_use_gpu;
            let cpu_border_color = if cpu_selected { ACCENT } else { CARD_BORDER };
            let cpu_bg = if cpu_selected { Color::from_rgba(0.39, 0.40, 0.95, 0.08) } else { CARD_BG };

            let cpu_card = container(
                column![
                    row![
                        text!("CPU").size(13).color(if cpu_selected { ACCENT } else { Color::WHITE }),
                        Space::with_width(Length::Fill),
                        if can_cpu {
                            text!("Compatible").size(9).color(SUCCESS)
                        } else {
                            text!("RAM insuffisante").size(9).color(DANGER)
                        },
                    ].spacing(6).align_y(iced::Alignment::Center),
                    text!("{}", self.hw.cpu.name).size(9).color(SUBTLE),
                    row![
                        text!("Requis:").size(9).color(MUTED),
                        text!("{:.1} Go RAM", reqs.cpu_ram_gb).size(9).color(Color::WHITE),
                        text!("|").size(9).color(CARD_BORDER),
                        text!("Dispo:").size(9).color(MUTED),
                        text!("{:.1} Go", cpu_ram_free).size(9).color(if can_cpu { SUCCESS } else { DANGER }),
                    ].spacing(4),
                    text!("{}", reqs.cpu_note).size(8).color(MUTED),
                ].spacing(4).padding(10)
            ).style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(cpu_bg)),
                border: border::rounded(8).color(cpu_border_color).width(if cpu_selected { 2 } else { 1 }),
                ..Default::default()
            });

            let cpu_btn = button(cpu_card).on_press(Message::WhisperUseGpu(false)).width(Length::Fill)
                .style(|_theme: &Theme, _status| button::Style::default());

            // GPU button
            let gpu_selected = self.whisper_use_gpu;
            let gpu_border_color = if gpu_selected { SUCCESS } else { CARD_BORDER };
            let gpu_bg = if gpu_selected { Color::from_rgba(0.20, 0.83, 0.60, 0.08) } else { CARD_BG };

            let gpu_inner = if gpu_available {
                let gpu = self.hw.gpu.as_ref().unwrap();
                column![
                    row![
                        text!("GPU").size(13).color(if gpu_selected { SUCCESS } else { Color::WHITE }),
                        Space::with_width(Length::Fill),
                        if can_gpu {
                            text!("Compatible").size(9).color(SUCCESS)
                        } else {
                            text!("VRAM insuffisante").size(9).color(DANGER)
                        },
                    ].spacing(6).align_y(iced::Alignment::Center),
                    text!("{}", gpu.name).size(9).color(SUBTLE),
                    row![
                        text!("Requis:").size(9).color(MUTED),
                        text!("{:.1} Go VRAM", reqs.gpu_vram_gb).size(9).color(Color::WHITE),
                        text!("|").size(9).color(CARD_BORDER),
                        text!("Dispo:").size(9).color(MUTED),
                        text!("{:.1} Go", gpu_vram_free).size(9).color(if can_gpu { SUCCESS } else { DANGER }),
                    ].spacing(4),
                    text!("{}", reqs.gpu_note).size(8).color(MUTED),
                ].spacing(4).padding(10)
            } else {
                column![
                    row![
                        text!("GPU").size(13).color(MUTED),
                        Space::with_width(Length::Fill),
                        text!("Non detecte").size(9).color(DANGER),
                    ].spacing(6).align_y(iced::Alignment::Center),
                    text!("Aucun GPU NVIDIA detecte").size(9).color(MUTED),
                ].spacing(4).padding(10)
            };

            let gpu_card = container(gpu_inner).style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(gpu_bg)),
                border: border::rounded(8).color(gpu_border_color).width(if gpu_selected { 2 } else { 1 }),
                ..Default::default()
            });

            let gpu_btn = if gpu_available {
                button(gpu_card).on_press(Message::WhisperUseGpu(true)).width(Length::Fill)
                    .style(|_theme: &Theme, _status| button::Style::default())
            } else {
                button(gpu_card).width(Length::Fill)
                    .style(|_theme: &Theme, _status| button::Style::default())
            };

            whisper_content = whisper_content
                .push(cpu_btn)
                .push(gpu_btn);

            // Recommendation
            let rec_text = match reqs.recommendation {
                hardware::DeviceRecommendation::CpuOnly => "CPU suffisant pour ce modele",
                hardware::DeviceRecommendation::GpuRecommended => "GPU recommande pour de meilleures performances",
                hardware::DeviceRecommendation::GpuRequired => "GPU fortement recommande pour ce modele",
            };
            let rec_color = match reqs.recommendation {
                hardware::DeviceRecommendation::CpuOnly => MUTED,
                hardware::DeviceRecommendation::GpuRecommended => ACCENT,
                hardware::DeviceRecommendation::GpuRequired => WARNING,
            };
            whisper_content = whisper_content.push(
                text(rec_text).size(9).color(rec_color)
            );
        }

        // Load button
        whisper_content = whisper_content.push(
            if self.loading_model {
                button(text!("...").size(12)).width(Length::Fill)
            } else {
                button(text(T::load(l)).size(12)).on_press(Message::LoadModel).style(button::primary).width(Length::Fill)
            }
        );

        if let Some(ref e) = self.load_error {
            whisper_content = whisper_content.push(text!("{e}").size(9).color(DANGER));
        }

        let whisper_card = container(whisper_content.padding(14)).style(card_style);

        // === Downloads Card ===
        let mut dl_content = column![
            row![
                text!("D").size(14).color(ACCENT),
                text(T::whisper_models(l)).size(14).color(Color::WHITE),
            ].spacing(8),
        ].spacing(10);

        for model in &self.all_models {
            let is_installed = models::is_model_installed(model);
            let name = model.name.clone();
            let size = format!("{}MB", model.size_mb);
            let params = model.params;
            let vram = model.vram_gpu;
            let ram = model.ram_cpu;

            dl_content = dl_content.push(
                column![
                    row![
                        if is_installed {
                            text!("ok").size(11).color(SUCCESS)
                        } else {
                            text!("-").size(11).color(MUTED)
                        },
                        column![
                            text!("{name}").size(11).color(if is_installed { Color::WHITE } else { SUBTLE }),
                            row![
                                text!("{params}").size(9).color(MUTED),
                                text!("·").size(9).color(CARD_BORDER),
                                text!("GPU {vram}").size(9).color(MUTED),
                                text!("·").size(9).color(CARD_BORDER),
                                text!("CPU {ram}").size(9).color(MUTED),
                            ].spacing(4),
                        ].spacing(2),
                        Space::with_width(Length::Fill),
                        text!("{size}").size(10).color(MUTED),
                        if !is_installed {
                            button(text(T::download(l)).size(9)).on_press(Message::DownloadModel(model.id.clone())).style(button::secondary)
                        } else {
                            button(text(T::installed(l)).size(9)).style(button::text)
                        },
                    ].spacing(8).align_y(iced::Alignment::Center),
                ].spacing(2)
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

        // === Fleet / Account Card ===
        let fleet_status_color = if self.fleet_connected { SUCCESS } else { MUTED };
        let fleet_status_text = if self.fleet_connected { "Connected" } else { "Not linked" };

        let mut fleet_content = column![
            row![
                text!("@").size(14).color(ACCENT),
                text!("Account").size(14).color(Color::WHITE),
                Space::with_width(Length::Fill),
                text!("*").size(8).color(fleet_status_color),
                text(fleet_status_text).size(10).color(fleet_status_color),
            ].spacing(6).align_y(iced::Alignment::Center),
        ].spacing(6);

        if !self.fleet_connected {
            fleet_content = fleet_content
                .push(
                    text_input("https://inkwell.example.com", &self.fleet_api_url)
                        .on_input(Message::FleetApiUrlChanged).size(11)
                )
                .push(
                    row![
                        text_input("Email", &self.fleet_email)
                            .on_input(Message::FleetEmailChanged).size(11),
                        text_input("Password", &self.fleet_password)
                            .on_input(Message::FleetPasswordChanged).size(11).secure(true),
                    ].spacing(6)
                )
                .push(
                    text_input("Node name (e.g. Bureau RTX 3060)", &self.fleet_node_name)
                        .on_input(Message::FleetNodeNameChanged).size(11)
                )
                .push(
                    if self.fleet_connecting {
                        button(text!("...").size(11))
                    } else {
                        button(text!("Connect").size(11)).on_press(Message::FleetConnect).style(button::primary)
                    }
                );
            if let Some(ref e) = self.fleet_error {
                fleet_content = fleet_content.push(text!("{e}").size(10).color(DANGER));
            }
        } else {
            fleet_content = fleet_content
                .push(text!("{}", self.fleet_node_name).size(11).color(Color::WHITE))
                .push(text!("{}", self.fleet_api_url).size(10).color(MUTED))
                .push(text!("{}", self.fleet_email).size(10).color(MUTED))
                .push(
                    button(text!("Disconnect").size(10)).on_press(Message::FleetDisconnect).style(button::secondary)
                );
        }

        let fleet_card = container(fleet_content.padding(14)).style(card_style);

        // === Log Card ===
        let mut log_content = column![
            text(T::activity(l)).size(12).color(SUBTLE),
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
                    fleet_card,
                    server_card,
                    hardware_card,
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
