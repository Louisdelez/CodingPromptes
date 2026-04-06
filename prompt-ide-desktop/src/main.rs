mod models;
mod services;
mod views;

use iced::widget::{button, column, container, horizontal_rule, row, text, text_editor, Space};
use iced::{Element, Length, Task, Theme};
use std::sync::Arc;
use tokio::sync::Mutex;

use models::block::{BlockType, PromptBlock};
use models::config::{available_models, AppConfig, ThemeMode};
use models::framework::builtin_frameworks;
use models::project::{PromptProject, PromptVersion, Workspace};
use services::backend::{BackendClient, BackendProject, BackendVersion, BackendWorkspace, AuthResponse};
use services::db::Database;
use services::i18n::I18n;

use views::auth::{AuthMessage, AuthMode, AuthView};
use views::editor::EditorMessage;
use views::library::LibraryMessage;
use views::playground::{PlaygroundMessage, PlaygroundResult};
use views::preview::PreviewMessage;
use views::settings::SettingsMessage;

fn main() -> iced::Result {
    iced::application("Prompt IDE Desktop", App::update, App::view)
        .theme(App::theme)
        .window_size((1200.0, 800.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum RightPanel { Preview, Playground, Settings }
#[derive(Debug, Clone)]
enum LeftPanel { Library, Frameworks, Versions }

#[derive(Debug, Clone)]
enum Message {
    Auth(AuthMessage),
    AuthResult(Result<AuthResponse, String>),
    Editor(EditorMessage),
    Library(LibraryMessage),
    Preview(PreviewMessage),
    Playground(PlaygroundMessage),
    PlaygroundResult(Result<services::api::ApiResponse, String>),
    Settings(SettingsMessage),
    SetLeftPanel(LeftPanel),
    SetRightPanel(RightPanel),
    SaveVersion,
    VersionLabelChanged(String),
    RestoreVersion(String),
    ApplyFramework(String),
    ProjectNameChanged(String),
    ExportTxt,
    ExportJson,
    // Async data results
    DataLoaded(Vec<Workspace>, Vec<PromptProject>),
    VersionsLoaded(Vec<PromptVersion>),
    ProjectSaved,
    ProjectCreated(PromptProject),
}

fn bp_to_local(bp: &BackendProject) -> PromptProject {
    PromptProject {
        id: bp.id.clone(), name: bp.name.clone(), user_id: bp.user_id.clone(),
        workspace_id: bp.workspace_id.clone(),
        blocks: serde_json::from_str(&bp.blocks_json).unwrap_or_default(),
        variables: serde_json::from_str(&bp.variables_json).unwrap_or_default(),
        framework: bp.framework.clone(),
        created_at: bp.created_at, updated_at: bp.updated_at,
    }
}

fn bw_to_local(bw: &BackendWorkspace) -> Workspace {
    Workspace {
        id: bw.id.clone(), name: bw.name.clone(), color: bw.color.clone(),
        user_id: bw.user_id.clone(), created_at: bw.created_at, updated_at: bw.updated_at,
    }
}

fn bv_to_local(bv: &BackendVersion) -> PromptVersion {
    PromptVersion {
        id: bv.id.clone(), project_id: bv.project_id.clone(),
        blocks_json: bv.blocks_json.clone(), variables_json: bv.variables_json.clone(),
        label: bv.label.clone(), created_at: bv.created_at,
    }
}

struct App {
    backend: Arc<Mutex<BackendClient>>,
    local_db: Database,
    i18n: I18n,
    config: AppConfig,
    session: Option<services::auth::Session>,
    auth_view: AuthView,
    workspaces: Vec<Workspace>,
    projects: Vec<PromptProject>,
    current_project: PromptProject,
    editor_contents: Vec<text_editor::Content>,
    versions: Vec<PromptVersion>,
    left_panel: LeftPanel,
    right_panel: RightPanel,
    search: String,
    new_ws_name: String,
    version_label: String,
    compiled_cache: String,
    selected_model: String,
    temperature: f32,
    max_tokens: f32,
    playground_results: Vec<PlaygroundResult>,
    executing: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let local_db = Database::open().expect("Failed to open local config database");
        let config = local_db.load_app_config();
        let i18n = I18n::new(&config.lang);
        let backend = Arc::new(Mutex::new(BackendClient::new(&config.local_server_url)));

        // Restore token from local config
        if let Some(token) = local_db.get_config("jwt_token") {
            let mut b = backend.try_lock().unwrap();
            b.set_token(token);
        }

        let current_project = PromptProject::new("", None);
        let editor_contents = current_project.blocks.iter()
            .map(|b| text_editor::Content::with_text(&b.content)).collect();

        let app = Self {
            backend, local_db, i18n, config,
            session: None, auth_view: AuthView::new(),
            workspaces: vec![], projects: vec![],
            current_project, editor_contents, versions: vec![],
            left_panel: LeftPanel::Library, right_panel: RightPanel::Preview,
            compiled_cache: String::new(), search: String::new(),
            new_ws_name: String::new(), version_label: String::new(),
            selected_model: "gpt-4o-mini".into(),
            temperature: 0.7, max_tokens: 2048.0,
            playground_results: vec![], executing: false,
        };

        // Try to restore session from saved token
        if app.local_db.get_config("jwt_token").is_some() {
            if let Some(name) = app.local_db.get_config("session_name") {
                // Restore session from local cache, will validate on data load
                let mut a = app;
                a.session = Some(services::auth::Session {
                    user_id: a.local_db.get_config("session_user_id").unwrap_or_default(),
                    email: a.local_db.get_config("session_email").unwrap_or_default(),
                    display_name: name,
                });
                let b = a.backend.clone();
                let task = Task::perform(async move { load_data(b).await }, |(ws, pj)| Message::DataLoaded(ws, pj));
                return (a, task);
            }
        }

        (app, Task::none())
    }

    fn sync_blocks_to_content(&mut self) {
        self.editor_contents = self.current_project.blocks.iter()
            .map(|b| text_editor::Content::with_text(&b.content)).collect();
    }

    fn save_project_async(&self) -> Task<Message> {
        let b = self.backend.clone();
        let p = self.current_project.clone();
        Task::perform(async move {
            let client = b.lock().await;
            let data = serde_json::json!({
                "name": p.name,
                "blocks_json": serde_json::to_string(&p.blocks).unwrap(),
                "variables_json": serde_json::to_string(&p.variables).unwrap(),
                "workspace_id": p.workspace_id,
                "framework": p.framework,
            });
            if client.update_project(&p.id, &data).await.is_err() {
                let create_data = serde_json::json!({
                    "id": p.id, "name": p.name,
                    "blocks_json": serde_json::to_string(&p.blocks).unwrap(),
                    "variables_json": serde_json::to_string(&p.variables).unwrap(),
                    "workspace_id": p.workspace_id,
                });
                client.create_project(&create_data).await.ok();
            }
        }, |_| Message::ProjectSaved)
    }

    fn load_data_task(&self) -> Task<Message> {
        let b = self.backend.clone();
        Task::perform(async move { load_data(b).await }, |(ws, pj)| Message::DataLoaded(ws, pj))
    }

    fn load_versions_task(&self) -> Task<Message> {
        let b = self.backend.clone();
        let pid = self.current_project.id.clone();
        Task::perform(async move {
            let client = b.lock().await;
            client.list_versions(&pid).await.unwrap_or_default().iter().map(bv_to_local).collect()
        }, Message::VersionsLoaded)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // === Auth ===
            Message::Auth(msg) => match msg {
                AuthMessage::EmailChanged(v) => { self.auth_view.email = v; }
                AuthMessage::PasswordChanged(v) => { self.auth_view.password = v; }
                AuthMessage::ConfirmPasswordChanged(v) => { self.auth_view.confirm_password = v; }
                AuthMessage::DisplayNameChanged(v) => { self.auth_view.display_name = v; }
                AuthMessage::SwitchMode => {
                    self.auth_view.mode = if self.auth_view.mode == AuthMode::Login { AuthMode::Register } else { AuthMode::Login };
                    self.auth_view.error = None;
                }
                AuthMessage::Submit => {
                    self.auth_view.error = None;
                    if self.auth_view.mode == AuthMode::Register {
                        if self.auth_view.password.len() < 6 {
                            self.auth_view.error = Some(self.i18n.t("auth.password_short").into());
                            return Task::none();
                        }
                        if self.auth_view.password != self.auth_view.confirm_password {
                            self.auth_view.error = Some(self.i18n.t("auth.password_mismatch").into());
                            return Task::none();
                        }
                        let b = self.backend.clone();
                        let email = self.auth_view.email.clone();
                        let pw = self.auth_view.password.clone();
                        let name = if self.auth_view.display_name.is_empty() {
                            email.split('@').next().unwrap_or("User").to_string()
                        } else { self.auth_view.display_name.clone() };
                        return Task::perform(async move {
                            let client = b.lock().await;
                            client.register(&email, &pw, &name).await
                        }, Message::AuthResult);
                    } else {
                        let b = self.backend.clone();
                        let email = self.auth_view.email.clone();
                        let pw = self.auth_view.password.clone();
                        return Task::perform(async move {
                            let client = b.lock().await;
                            client.login(&email, &pw).await
                        }, Message::AuthResult);
                    }
                }
            },

            Message::AuthResult(result) => {
                match result {
                    Ok(resp) => {
                        // Save token
                        {
                            let mut b = self.backend.try_lock().unwrap();
                            b.set_token(resp.token.clone());
                        }
                        self.local_db.set_config("jwt_token", &resp.token);
                        self.local_db.set_config("session_user_id", &resp.user.id);
                        self.local_db.set_config("session_email", &resp.user.email);
                        self.local_db.set_config("session_name", &resp.user.display_name);

                        self.session = Some(services::auth::Session {
                            user_id: resp.user.id, email: resp.user.email, display_name: resp.user.display_name,
                        });
                        return self.load_data_task();
                    }
                    Err(e) => {
                        self.auth_view.error = Some(if e.contains("INVALID_CREDENTIALS") {
                            self.i18n.t("auth.invalid_credentials").into()
                        } else if e.contains("EMAIL_EXISTS") {
                            self.i18n.t("auth.email_exists").into()
                        } else { e });
                    }
                }
            }

            // === Data loaded ===
            Message::DataLoaded(ws, pj) => {
                self.workspaces = ws;
                self.projects = pj;
                if let Some(p) = self.projects.first().cloned() {
                    self.current_project = p;
                } else {
                    self.current_project = PromptProject::new(
                        self.session.as_ref().map(|s| s.user_id.as_str()).unwrap_or(""), None);
                }
                self.sync_blocks_to_content();
                self.compiled_cache = self.current_project.compile();
                return self.load_versions_task();
            }

            Message::VersionsLoaded(vs) => { self.versions = vs; }
            Message::ProjectSaved => {}
            Message::ProjectCreated(p) => {
                self.current_project = p;
                self.sync_blocks_to_content();
                self.compiled_cache = self.current_project.compile();
                return self.load_data_task();
            }

            // === Editor ===
            Message::Editor(msg) => match msg {
                EditorMessage::BlockContentChanged(i, action) => {
                    if let Some(content) = self.editor_contents.get_mut(i) {
                        content.perform(action);
                        self.current_project.blocks[i].content = content.text();
                        self.compiled_cache = self.current_project.compile();
                        return self.save_project_async();
                    }
                }
                EditorMessage::ToggleBlock(i) => {
                    self.current_project.blocks[i].enabled = !self.current_project.blocks[i].enabled;
                    self.compiled_cache = self.current_project.compile();
                    return self.save_project_async();
                }
                EditorMessage::RemoveBlock(i) => {
                    self.current_project.blocks.remove(i);
                    self.editor_contents.remove(i);
                    self.compiled_cache = self.current_project.compile();
                    return self.save_project_async();
                }
                EditorMessage::AddBlock(bt) => {
                    self.current_project.blocks.push(PromptBlock::new(bt));
                    self.editor_contents.push(text_editor::Content::with_text(""));
                    return self.save_project_async();
                }
                EditorMessage::MoveBlockUp(i) => {
                    if i > 0 {
                        self.current_project.blocks.swap(i, i - 1);
                        self.editor_contents.swap(i, i - 1);
                        return self.save_project_async();
                    }
                }
                EditorMessage::MoveBlockDown(i) => {
                    if i + 1 < self.current_project.blocks.len() {
                        self.current_project.blocks.swap(i, i + 1);
                        self.editor_contents.swap(i, i + 1);
                        return self.save_project_async();
                    }
                }
            },

            // === Library ===
            Message::Library(msg) => match msg {
                LibraryMessage::SearchChanged(s) => self.search = s,
                LibraryMessage::SelectProject(id) => {
                    if let Some(p) = self.projects.iter().find(|p| p.id == id).cloned() {
                        self.current_project = p;
                        self.sync_blocks_to_content();
                        self.compiled_cache = self.current_project.compile();
                        return self.load_versions_task();
                    }
                }
                LibraryMessage::NewProject(ws_id) => {
                    let b = self.backend.clone();
                    let uid = self.session.as_ref().map(|s| s.user_id.clone()).unwrap_or_default();
                    return Task::perform(async move {
                        let client = b.lock().await;
                        let data = serde_json::json!({
                            "name": "Nouveau prompt",
                            "blocks_json": "[]",
                            "variables_json": "{}",
                            "workspace_id": ws_id,
                        });
                        let bp = client.create_project(&data).await.ok();
                        bp.map(|p| bp_to_local(&p)).unwrap_or_else(|| PromptProject::new(&uid, None))
                    }, Message::ProjectCreated);
                }
                LibraryMessage::DeleteProject(id) => {
                    let b = self.backend.clone();
                    let b2 = self.backend.clone();
                    return Task::perform(async move {
                        let client = b.lock().await;
                        client.delete_project(&id).await.ok();
                        drop(client);
                        load_data(b2).await
                    }, |(ws, pj)| Message::DataLoaded(ws, pj));
                }
                LibraryMessage::DuplicateProject(id) => {
                    if let Some(p) = self.projects.iter().find(|p| p.id == id) {
                        let b = self.backend.clone();
                        let name = format!("{} (copie)", p.name);
                        let blocks = serde_json::to_string(&p.blocks).unwrap();
                        let vars = serde_json::to_string(&p.variables).unwrap();
                        let ws = p.workspace_id.clone();
                        return Task::perform(async move {
                            let client = b.lock().await;
                            client.create_project(&serde_json::json!({
                                "name": name, "blocks_json": blocks, "variables_json": vars, "workspace_id": ws,
                            })).await.ok();
                        }, |_| Message::ProjectSaved).chain(self.load_data_task());
                    }
                }
                LibraryMessage::NewWorkspace => { self.new_ws_name = "Nouveau projet".into(); }
                LibraryMessage::WorkspaceNameChanged(s) => self.new_ws_name = s,
                LibraryMessage::CreateWorkspace => {
                    if !self.new_ws_name.is_empty() {
                        let b = self.backend.clone();
                        let name = self.new_ws_name.clone();
                        self.new_ws_name.clear();
                        return Task::perform(async move {
                            let client = b.lock().await;
                            client.create_workspace(&name, "#6366f1").await.ok();
                        }, |_| Message::ProjectSaved).chain(self.load_data_task());
                    }
                }
                LibraryMessage::DeleteWorkspace(id) => {
                    let b = self.backend.clone();
                    return Task::perform(async move {
                        let client = b.lock().await;
                        client.delete_workspace(&id).await.ok();
                    }, |_| Message::ProjectSaved).chain(self.load_data_task());
                }
            },

            Message::Preview(PreviewMessage::Copy) => {}

            // === Playground ===
            Message::Playground(msg) => match msg {
                PlaygroundMessage::SelectModel(m) => self.selected_model = m,
                PlaygroundMessage::TemperatureChanged(t) => self.temperature = t,
                PlaygroundMessage::MaxTokensChanged(t) => self.max_tokens = t,
                PlaygroundMessage::Execute => {
                    let compiled = self.current_project.compile();
                    if compiled.is_empty() || self.executing { return Task::none(); }
                    self.executing = true;
                    self.playground_results.clear();
                    let model_id = self.selected_model.clone();
                    let models = available_models();
                    let provider = models.iter().find(|m| m.id == model_id).map(|m| m.provider).unwrap_or("openai").to_string();
                    let config = self.config.clone();
                    let temp = self.temperature;
                    let max_tok = self.max_tokens as u32;
                    return Task::perform(async move {
                        services::api::call_llm(&compiled, &model_id, &provider, &config, temp, max_tok).await
                    }, Message::PlaygroundResult);
                }
            },

            Message::PlaygroundResult(result) => {
                self.executing = false;
                match result {
                    Ok(resp) => self.playground_results.push(PlaygroundResult {
                        model: self.selected_model.clone(), response: resp.text,
                        tokens_in: resp.tokens_in, tokens_out: resp.tokens_out,
                        latency_ms: resp.latency_ms, error: None,
                    }),
                    Err(e) => self.playground_results.push(PlaygroundResult {
                        model: self.selected_model.clone(), response: String::new(),
                        tokens_in: 0, tokens_out: 0, latency_ms: 0, error: Some(e),
                    }),
                }
            }

            // === Settings ===
            Message::Settings(msg) => {
                match msg {
                    SettingsMessage::OpenAiKeyChanged(v) => self.config.openai_key = v,
                    SettingsMessage::AnthropicKeyChanged(v) => self.config.anthropic_key = v,
                    SettingsMessage::GoogleKeyChanged(v) => self.config.google_key = v,
                    SettingsMessage::GroqKeyChanged(v) => self.config.groq_key = v,
                    SettingsMessage::LocalServerUrlChanged(v) => {
                        self.config.local_server_url = v.clone();
                        if let Ok(mut b) = self.backend.try_lock() { b.set_base_url(&v); }
                    }
                    SettingsMessage::ToggleTheme(dark) => {
                        self.config.theme = if dark { ThemeMode::Dark } else { ThemeMode::Light };
                    }
                    SettingsMessage::ToggleLang(en) => {
                        self.config.lang = if en { "en" } else { "fr" }.into();
                        self.i18n.set_lang(&self.config.lang);
                    }
                    SettingsMessage::Logout => {
                        self.session = None;
                        self.auth_view = AuthView::new();
                        if let Ok(mut b) = self.backend.try_lock() { b.clear_token(); }
                        self.local_db.set_config("jwt_token", "");
                    }
                }
                self.local_db.save_app_config(&self.config);
            }

            Message::SetLeftPanel(p) => self.left_panel = p,
            Message::SetRightPanel(p) => self.right_panel = p,

            // === Versions ===
            Message::VersionLabelChanged(s) => self.version_label = s,
            Message::SaveVersion => {
                if !self.version_label.is_empty() {
                    let b = self.backend.clone();
                    let pid = self.current_project.id.clone();
                    let blocks = serde_json::to_string(&self.current_project.blocks).unwrap();
                    let vars = serde_json::to_string(&self.current_project.variables).unwrap();
                    let label = self.version_label.clone();
                    self.version_label.clear();
                    return Task::perform(async move {
                        let client = b.lock().await;
                        client.create_version(&pid, &serde_json::json!({
                            "blocks_json": blocks, "variables_json": vars, "label": label,
                        })).await.ok();
                    }, |_| Message::ProjectSaved).chain(self.load_versions_task());
                }
            }
            Message::RestoreVersion(id) => {
                if let Some(v) = self.versions.iter().find(|v| v.id == id) {
                    if let Ok(blocks) = serde_json::from_str::<Vec<PromptBlock>>(&v.blocks_json) {
                        self.current_project.blocks = blocks;
                        if let Ok(vars) = serde_json::from_str(&v.variables_json) {
                            self.current_project.variables = vars;
                        }
                        self.sync_blocks_to_content();
                        self.compiled_cache = self.current_project.compile();
                        return self.save_project_async();
                    }
                }
            }

            Message::ApplyFramework(id) => {
                if let Some(fw) = builtin_frameworks().into_iter().find(|f| f.id == id) {
                    self.current_project.blocks = fw.to_blocks();
                    self.current_project.framework = Some(fw.id);
                    self.sync_blocks_to_content();
                    self.compiled_cache = self.current_project.compile();
                    return self.save_project_async();
                }
            }

            Message::ProjectNameChanged(name) => {
                self.current_project.name = name;
                return self.save_project_async().chain(self.load_data_task());
            }

            Message::ExportTxt => {
                let compiled = self.current_project.compile();
                let path = dirs::document_dir().unwrap_or_default().join(format!("{}.txt", self.current_project.name));
                std::fs::write(&path, &compiled).ok();
            }
            Message::ExportJson => {
                let json = serde_json::to_string_pretty(&self.current_project).unwrap();
                let path = dirs::document_dir().unwrap_or_default().join(format!("{}.json", self.current_project.name));
                std::fs::write(&path, &json).ok();
            }
        }
        Task::none()
    }

    fn theme(&self) -> Theme {
        match self.config.theme { ThemeMode::Dark => Theme::Dark, ThemeMode::Light => Theme::Light }
    }

    fn view(&self) -> Element<Message> {
        if self.session.is_none() {
            return self.auth_view.view(&self.i18n).map(Message::Auth);
        }
        let session = self.session.as_ref().unwrap();

        let header = container(row![
            text("Prompt IDE").size(16), Space::with_width(8),
            iced::widget::text_input("Nom...", &self.current_project.name)
                .on_input(Message::ProjectNameChanged).size(13).width(Length::Fixed(200.0)),
            Space::with_width(Length::Fill),
            text(format!("👤 {}", session.display_name)).size(12),
        ].align_y(iced::Alignment::Center).padding(8));

        let left_tabs = row![
            button(text(self.i18n.t("tab.library")).size(11)).on_press(Message::SetLeftPanel(LeftPanel::Library))
                .style(if matches!(self.left_panel, LeftPanel::Library) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.frameworks")).size(11)).on_press(Message::SetLeftPanel(LeftPanel::Frameworks))
                .style(if matches!(self.left_panel, LeftPanel::Frameworks) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.versions")).size(11)).on_press(Message::SetLeftPanel(LeftPanel::Versions))
                .style(if matches!(self.left_panel, LeftPanel::Versions) { button::primary } else { button::secondary }),
        ].spacing(2);

        let left_content: Element<Message> = match self.left_panel {
            LeftPanel::Library => views::library::view_library(
                &self.workspaces, &self.projects, &self.current_project.id,
                &self.search, &self.new_ws_name, &self.i18n,
            ).map(Message::Library),
            LeftPanel::Frameworks => {
                let mut col = column![text(self.i18n.t("frameworks.title")).size(14)].spacing(6).padding(8);
                for fw in builtin_frameworks() {
                    let name = fw.name.clone(); let desc = fw.description.clone();
                    col = col.push(button(column![
                        text(name).size(13),
                        text(desc).size(10).color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
                    ].spacing(2)).on_press(Message::ApplyFramework(fw.id.clone())).width(Length::Fill).style(button::secondary));
                }
                iced::widget::scrollable(col).height(Length::Fill).into()
            }
            LeftPanel::Versions => {
                let mut col = column![
                    text(self.i18n.t("versions.title")).size(14),
                    row![
                        iced::widget::text_input(self.i18n.t("versions.label"), &self.version_label)
                            .on_input(Message::VersionLabelChanged).size(12),
                        button(text(self.i18n.t("versions.save")).size(11)).on_press(Message::SaveVersion).style(button::primary),
                    ].spacing(4),
                ].spacing(8).padding(8);
                if self.versions.is_empty() {
                    col = col.push(text(self.i18n.t("versions.empty")).size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.55)));
                }
                for v in &self.versions {
                    col = col.push(row![
                        text(&v.label).size(12), Space::with_width(Length::Fill),
                        button(text(self.i18n.t("versions.restore")).size(10)).on_press(Message::RestoreVersion(v.id.clone())).style(button::secondary),
                    ].align_y(iced::Alignment::Center));
                }
                iced::widget::scrollable(col).height(Length::Fill).into()
            }
        };

        let left_panel = column![left_tabs, left_content].spacing(4).width(Length::Fixed(260.0));

        let center = views::editor::view_blocks(&self.current_project.blocks, &self.editor_contents, &self.i18n).map(Message::Editor);
        let compiled = &self.compiled_cache;
        let chars = compiled.len();
        let words = if compiled.is_empty() { 0 } else { compiled.split_whitespace().count() };
        let lines = if compiled.is_empty() { 0 } else { compiled.lines().count() };
        let tokens_est = chars / 4;
        let counter = container(row![
            text(format!("{chars} {}", self.i18n.t("counter.chars"))).size(11),
            text(format!("{words} {}", self.i18n.t("counter.words"))).size(11),
            text(format!("{lines} {}", self.i18n.t("counter.lines"))).size(11),
            text(format!("~{tokens_est} {}", self.i18n.t("counter.tokens"))).size(11),
        ].spacing(12).padding(6));
        let center_panel = column![center, horizontal_rule(1), counter].width(Length::Fill);

        let right_tabs = row![
            button(text(self.i18n.t("tab.preview")).size(11)).on_press(Message::SetRightPanel(RightPanel::Preview))
                .style(if matches!(self.right_panel, RightPanel::Preview) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.playground")).size(11)).on_press(Message::SetRightPanel(RightPanel::Playground))
                .style(if matches!(self.right_panel, RightPanel::Playground) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.settings")).size(11)).on_press(Message::SetRightPanel(RightPanel::Settings))
                .style(if matches!(self.right_panel, RightPanel::Settings) { button::primary } else { button::secondary }),
        ].spacing(2);

        let right_content: Element<Message> = match self.right_panel {
            RightPanel::Preview => views::preview::view_preview(compiled, &self.i18n).map(Message::Preview),
            RightPanel::Playground => views::playground::view_playground(
                &self.selected_model, self.temperature, self.max_tokens,
                &self.playground_results, self.executing, &self.i18n,
            ).map(Message::Playground),
            RightPanel::Settings => views::settings::view_settings(&self.config, &session.display_name, &self.i18n).map(Message::Settings),
        };

        let right_panel = column![right_tabs, right_content].spacing(4).width(Length::Fixed(360.0));
        let main = row![left_panel, center_panel, right_panel].spacing(1).height(Length::Fill);

        container(column![header, horizontal_rule(1), main]).width(Length::Fill).height(Length::Fill).into()
    }
}

async fn load_data(b: Arc<Mutex<BackendClient>>) -> (Vec<Workspace>, Vec<PromptProject>) {
    let client = b.lock().await;
    let ws = client.list_workspaces().await.unwrap_or_default().iter().map(bw_to_local).collect();
    let pj = client.list_projects().await.unwrap_or_default().iter().map(bp_to_local).collect();
    (ws, pj)
}
