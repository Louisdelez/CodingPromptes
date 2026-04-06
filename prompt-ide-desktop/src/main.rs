mod models;
mod services;
mod views;

use iced::widget::{button, column, container, horizontal_rule, row, text, text_editor, Space};
use iced::{Element, Length, Task, Theme};

use models::block::{BlockType, PromptBlock};
use models::config::{available_models, AppConfig, ThemeMode};
use models::framework::builtin_frameworks;
use models::project::{ExecutionResult, PromptProject, PromptVersion, Workspace};
use services::auth::Session;
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
enum RightPanel {
    Preview,
    Playground,
    Settings,
}

#[derive(Debug, Clone)]
enum LeftPanel {
    Library,
    Frameworks,
    Versions,
}

#[derive(Debug, Clone)]
enum Message {
    // Auth
    Auth(AuthMessage),
    // Editor
    Editor(EditorMessage),
    // Library
    Library(LibraryMessage),
    // Preview
    Preview(PreviewMessage),
    // Playground
    Playground(PlaygroundMessage),
    PlaygroundResult(Result<services::api::ApiResponse, String>),
    // Settings
    Settings(SettingsMessage),
    // Navigation
    SetLeftPanel(LeftPanel),
    SetRightPanel(RightPanel),
    // Versions
    SaveVersion,
    VersionLabelChanged(String),
    RestoreVersion(String),
    // Frameworks
    ApplyFramework(String),
    // Project name
    ProjectNameChanged(String),
    // Export
    ExportTxt,
    ExportJson,
}

struct App {
    db: Database,
    i18n: I18n,
    config: AppConfig,

    // Auth
    session: Option<Session>,
    auth_view: AuthView,

    // Data
    workspaces: Vec<Workspace>,
    projects: Vec<PromptProject>,
    current_project: PromptProject,
    editor_contents: Vec<text_editor::Content>,
    versions: Vec<PromptVersion>,

    // UI state
    left_panel: LeftPanel,
    right_panel: RightPanel,
    search: String,
    new_ws_name: String,
    version_label: String,

    // Cache
    compiled_cache: String,

    // Playground
    selected_model: String,
    temperature: f32,
    max_tokens: f32,
    playground_results: Vec<PlaygroundResult>,
    executing: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let db = Database::open().expect("Failed to open database");
        let config = db.load_app_config();
        let i18n = I18n::new(&config.lang);

        let current_project = PromptProject::new("", None);
        let editor_contents = current_project
            .blocks
            .iter()
            .map(|b| text_editor::Content::with_text(&b.content))
            .collect();

        let app = Self {
            db,
            i18n,
            config,
            session: None,
            auth_view: AuthView::new(),
            workspaces: vec![],
            projects: vec![],
            current_project,
            editor_contents,
            versions: vec![],
            left_panel: LeftPanel::Library,
            right_panel: RightPanel::Preview,
            compiled_cache: String::new(),
            search: String::new(),
            new_ws_name: String::new(),
            version_label: String::new(),
            selected_model: "gpt-4o-mini".into(),
            temperature: 0.7,
            max_tokens: 2048.0,
            playground_results: vec![],
            executing: false,
        };

        (app, Task::none())
    }

    fn reload_data(&mut self) {
        if let Some(ref session) = self.session {
            self.workspaces = self.db.list_workspaces(&session.user_id);
            self.projects = self.db.list_projects(&session.user_id);
            self.versions = self.db.list_versions(&self.current_project.id);
        }
    }

    fn sync_blocks_to_content(&mut self) {
        self.editor_contents = self.current_project.blocks.iter()
            .map(|b| text_editor::Content::with_text(&b.content))
            .collect();
    }

    fn save_current_project(&mut self) {
        self.current_project.updated_at = chrono::Utc::now().timestamp_millis();
        self.compiled_cache = self.current_project.compile();
        self.db.save_project(&self.current_project).ok();
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // === Auth ===
            Message::Auth(msg) => {
                match msg {
                    AuthMessage::EmailChanged(v) => self.auth_view.email = v,
                    AuthMessage::PasswordChanged(v) => self.auth_view.password = v,
                    AuthMessage::ConfirmPasswordChanged(v) => self.auth_view.confirm_password = v,
                    AuthMessage::DisplayNameChanged(v) => self.auth_view.display_name = v,
                    AuthMessage::SwitchMode => {
                        self.auth_view.mode = if self.auth_view.mode == AuthMode::Login {
                            AuthMode::Register
                        } else {
                            AuthMode::Login
                        };
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
                            let name = if self.auth_view.display_name.is_empty() {
                                self.auth_view.email.split('@').next().unwrap_or("User").to_string()
                            } else {
                                self.auth_view.display_name.clone()
                            };
                            match self.db.register(&self.auth_view.email, &self.auth_view.password, &name) {
                                Ok(user) => {
                                    self.session = Some(Session {
                                        user_id: user.id, email: user.email, display_name: user.display_name,
                                    });
                                    self.current_project = PromptProject::new(&self.session.as_ref().unwrap().user_id, None);
                                    self.sync_blocks_to_content();
                                    self.reload_data();
                                }
                                Err(e) => {
                                    self.auth_view.error = Some(match e.as_str() {
                                        "EMAIL_EXISTS" => self.i18n.t("auth.email_exists").into(),
                                        _ => e,
                                    });
                                }
                            }
                        } else {
                            match self.db.login(&self.auth_view.email, &self.auth_view.password) {
                                Ok(user) => {
                                    self.session = Some(Session {
                                        user_id: user.id.clone(), email: user.email, display_name: user.display_name,
                                    });
                                    self.reload_data();
                                    // Load most recent project
                                    if let Some(p) = self.projects.first().cloned() {
                                        self.current_project = p;
                                    } else {
                                        self.current_project = PromptProject::new(&user.id, None);
                                    }
                                    self.sync_blocks_to_content();
                                    self.versions = self.db.list_versions(&self.current_project.id);
                                }
                                Err(e) => {
                                    self.auth_view.error = Some(match e.as_str() {
                                        "INVALID_CREDENTIALS" => self.i18n.t("auth.invalid_credentials").into(),
                                        _ => e,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // === Editor ===
            Message::Editor(msg) => match msg {
                EditorMessage::BlockContentChanged(i, action) => {
                    if let Some(content) = self.editor_contents.get_mut(i) {
                        content.perform(action);
                        self.current_project.blocks[i].content = content.text();
                        self.save_current_project();
                    }
                }
                EditorMessage::ToggleBlock(i) => {
                    self.current_project.blocks[i].enabled = !self.current_project.blocks[i].enabled;
                    self.save_current_project();
                }
                EditorMessage::RemoveBlock(i) => {
                    self.current_project.blocks.remove(i);
                    self.editor_contents.remove(i);
                    self.save_current_project();
                }
                EditorMessage::AddBlock(bt) => {
                    let block = PromptBlock::new(bt);
                    self.editor_contents.push(text_editor::Content::with_text(""));
                    self.current_project.blocks.push(block);
                    self.save_current_project();
                }
                EditorMessage::MoveBlockUp(i) => {
                    if i > 0 {
                        self.current_project.blocks.swap(i, i - 1);
                        self.editor_contents.swap(i, i - 1);
                        self.save_current_project();
                    }
                }
                EditorMessage::MoveBlockDown(i) => {
                    if i + 1 < self.current_project.blocks.len() {
                        self.current_project.blocks.swap(i, i + 1);
                        self.editor_contents.swap(i, i + 1);
                        self.save_current_project();
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
                        self.versions = self.db.list_versions(&self.current_project.id);
                    }
                }
                LibraryMessage::NewProject(ws_id) => {
                    let uid = self.session.as_ref().map(|s| s.user_id.as_str()).unwrap_or("");
                    self.current_project = PromptProject::new(uid, ws_id);
                    self.sync_blocks_to_content();
                    self.save_current_project();
                    self.reload_data();
                }
                LibraryMessage::DeleteProject(id) => {
                    self.db.delete_project(&id).ok();
                    self.reload_data();
                }
                LibraryMessage::DuplicateProject(id) => {
                    if let Some(p) = self.projects.iter().find(|p| p.id == id) {
                        let mut copy = p.clone();
                        copy.id = uuid::Uuid::new_v4().to_string();
                        copy.name = format!("{} (copie)", p.name);
                        copy.created_at = chrono::Utc::now().timestamp_millis();
                        copy.updated_at = copy.created_at;
                        self.db.save_project(&copy).ok();
                        self.reload_data();
                    }
                }
                LibraryMessage::NewWorkspace => {
                    self.new_ws_name = "Nouveau projet".into();
                }
                LibraryMessage::WorkspaceNameChanged(s) => self.new_ws_name = s,
                LibraryMessage::CreateWorkspace => {
                    if !self.new_ws_name.is_empty() {
                        let uid = self.session.as_ref().map(|s| s.user_id.as_str()).unwrap_or("");
                        let ws = Workspace {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: self.new_ws_name.clone(),
                            color: "#6366f1".into(),
                            user_id: uid.into(),
                            created_at: chrono::Utc::now().timestamp_millis(),
                            updated_at: chrono::Utc::now().timestamp_millis(),
                        };
                        self.db.create_workspace(&ws).ok();
                        self.new_ws_name.clear();
                        self.reload_data();
                    }
                }
                LibraryMessage::DeleteWorkspace(id) => {
                    self.db.delete_workspace(&id).ok();
                    self.reload_data();
                }
            },

            // === Preview ===
            Message::Preview(PreviewMessage::Copy) => {
                // Iced doesn't have clipboard API directly, but the compiled text is shown
            }

            // === Playground ===
            Message::Playground(msg) => match msg {
                PlaygroundMessage::SelectModel(m) => self.selected_model = m,
                PlaygroundMessage::TemperatureChanged(t) => self.temperature = t,
                PlaygroundMessage::MaxTokensChanged(t) => self.max_tokens = t,
                PlaygroundMessage::Execute => {
                    let compiled = self.current_project.compile();
                    if compiled.is_empty() || self.executing {
                        return Task::none();
                    }
                    self.executing = true;
                    self.playground_results.clear();

                    let model_id = self.selected_model.clone();
                    let models = available_models();
                    let provider = models.iter().find(|m| m.id == model_id).map(|m| m.provider).unwrap_or("openai").to_string();
                    let config = self.config.clone();
                    let temp = self.temperature;
                    let max_tok = self.max_tokens as u32;

                    return Task::perform(
                        async move {
                            services::api::call_llm(&compiled, &model_id, &provider, &config, temp, max_tok).await
                        },
                        Message::PlaygroundResult,
                    );
                }
            },

            Message::PlaygroundResult(result) => {
                self.executing = false;
                match result {
                    Ok(resp) => {
                        self.playground_results.push(PlaygroundResult {
                            model: self.selected_model.clone(),
                            response: resp.text,
                            tokens_in: resp.tokens_in,
                            tokens_out: resp.tokens_out,
                            latency_ms: resp.latency_ms,
                            error: None,
                        });
                    }
                    Err(e) => {
                        self.playground_results.push(PlaygroundResult {
                            model: self.selected_model.clone(),
                            response: String::new(),
                            tokens_in: 0,
                            tokens_out: 0,
                            latency_ms: 0,
                            error: Some(e),
                        });
                    }
                }
            }

            // === Settings ===
            Message::Settings(msg) => {
                match msg {
                    SettingsMessage::OpenAiKeyChanged(v) => self.config.openai_key = v,
                    SettingsMessage::AnthropicKeyChanged(v) => self.config.anthropic_key = v,
                    SettingsMessage::GoogleKeyChanged(v) => self.config.google_key = v,
                    SettingsMessage::GroqKeyChanged(v) => self.config.groq_key = v,
                    SettingsMessage::LocalServerUrlChanged(v) => self.config.local_server_url = v,
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
                    }
                }
                self.db.save_app_config(&self.config);
            }

            // === Navigation ===
            Message::SetLeftPanel(p) => self.left_panel = p,
            Message::SetRightPanel(p) => self.right_panel = p,

            // === Versions ===
            Message::VersionLabelChanged(s) => self.version_label = s,
            Message::SaveVersion => {
                if !self.version_label.is_empty() {
                    let v = PromptVersion {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: self.current_project.id.clone(),
                        blocks_json: serde_json::to_string(&self.current_project.blocks).unwrap(),
                        variables_json: serde_json::to_string(&self.current_project.variables).unwrap(),
                        label: self.version_label.clone(),
                        created_at: chrono::Utc::now().timestamp_millis(),
                    };
                    self.db.save_version(&v).ok();
                    self.version_label.clear();
                    self.versions = self.db.list_versions(&self.current_project.id);
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
                        self.save_current_project();
                    }
                }
            }

            // === Frameworks ===
            Message::ApplyFramework(id) => {
                let frameworks = builtin_frameworks();
                if let Some(fw) = frameworks.iter().find(|f| f.id == id) {
                    self.current_project.blocks = fw.to_blocks();
                    self.current_project.framework = Some(fw.id.clone());
                    self.sync_blocks_to_content();
                    self.save_current_project();
                }
            }

            // === Project name ===
            Message::ProjectNameChanged(name) => {
                self.current_project.name = name;
                self.save_current_project();
                self.reload_data();
            }

            // === Export ===
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
        match self.config.theme {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
        }
    }

    fn view(&self) -> Element<Message> {
        if self.session.is_none() {
            return self.auth_view.view(&self.i18n).map(Message::Auth);
        }

        let session = self.session.as_ref().unwrap();

        // Header
        let header = container(
            row![
                text("Prompt IDE").size(16),
                Space::with_width(8),
                iced::widget::text_input("Nom...", &self.current_project.name)
                    .on_input(Message::ProjectNameChanged)
                    .size(13)
                    .width(Length::Fixed(200.0)),
                Space::with_width(Length::Fill),
                text(format!("👤 {}", session.display_name)).size(12),
            ]
            .align_y(iced::Alignment::Center)
            .padding(8),
        );

        // Left panel tabs
        let left_tabs = row![
            button(text(self.i18n.t("tab.library")).size(11))
                .on_press(Message::SetLeftPanel(LeftPanel::Library))
                .style(if matches!(self.left_panel, LeftPanel::Library) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.frameworks")).size(11))
                .on_press(Message::SetLeftPanel(LeftPanel::Frameworks))
                .style(if matches!(self.left_panel, LeftPanel::Frameworks) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.versions")).size(11))
                .on_press(Message::SetLeftPanel(LeftPanel::Versions))
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
                    let name = fw.name.clone();
                    let desc = fw.description.clone();
                    col = col.push(
                        button(column![
                            text(name).size(13),
                            text(desc).size(10).color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
                        ].spacing(2))
                        .on_press(Message::ApplyFramework(fw.id.clone()))
                        .width(Length::Fill)
                        .style(button::secondary),
                    );
                }
                iced::widget::scrollable(col).height(Length::Fill).into()
            }
            LeftPanel::Versions => {
                let mut col = column![
                    text(self.i18n.t("versions.title")).size(14),
                    row![
                        iced::widget::text_input(self.i18n.t("versions.label"), &self.version_label)
                            .on_input(Message::VersionLabelChanged)
                            .size(12),
                        button(text(self.i18n.t("versions.save")).size(11))
                            .on_press(Message::SaveVersion)
                            .style(button::primary),
                    ].spacing(4),
                ].spacing(8).padding(8);

                if self.versions.is_empty() {
                    col = col.push(text(self.i18n.t("versions.empty")).size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.55)));
                }
                for v in &self.versions {
                    col = col.push(
                        row![
                            text(&v.label).size(12),
                            Space::with_width(Length::Fill),
                            button(text(self.i18n.t("versions.restore")).size(10))
                                .on_press(Message::RestoreVersion(v.id.clone()))
                                .style(button::secondary),
                        ].align_y(iced::Alignment::Center),
                    );
                }
                iced::widget::scrollable(col).height(Length::Fill).into()
            }
        };

        let left_panel = column![left_tabs, left_content].spacing(4).width(Length::Fixed(260.0));

        // Center: editor
        let center = views::editor::view_blocks(
            &self.current_project.blocks,
            &self.editor_contents,
            &self.i18n,
        ).map(Message::Editor);

        // Counter bar
        let compiled = &self.compiled_cache;
        let chars = compiled.len();
        let words = if compiled.is_empty() { 0 } else { compiled.split_whitespace().count() };
        let lines = if compiled.is_empty() { 0 } else { compiled.lines().count() };
        let tokens_est = chars / 4;

        let counter = container(
            row![
                text(format!("{chars} {}", self.i18n.t("counter.chars"))).size(11),
                text(format!("{words} {}", self.i18n.t("counter.words"))).size(11),
                text(format!("{lines} {}", self.i18n.t("counter.lines"))).size(11),
                text(format!("~{tokens_est} {}", self.i18n.t("counter.tokens"))).size(11),
            ]
            .spacing(12)
            .padding(6),
        );

        let center_panel = column![center, horizontal_rule(1), counter].width(Length::Fill);

        // Right panel tabs
        let right_tabs = row![
            button(text(self.i18n.t("tab.preview")).size(11))
                .on_press(Message::SetRightPanel(RightPanel::Preview))
                .style(if matches!(self.right_panel, RightPanel::Preview) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.playground")).size(11))
                .on_press(Message::SetRightPanel(RightPanel::Playground))
                .style(if matches!(self.right_panel, RightPanel::Playground) { button::primary } else { button::secondary }),
            button(text(self.i18n.t("tab.settings")).size(11))
                .on_press(Message::SetRightPanel(RightPanel::Settings))
                .style(if matches!(self.right_panel, RightPanel::Settings) { button::primary } else { button::secondary }),
        ].spacing(2);

        let right_content: Element<Message> = match self.right_panel {
            RightPanel::Preview => {
                views::preview::view_preview(&compiled, &self.i18n).map(Message::Preview)
            }
            RightPanel::Playground => {
                views::playground::view_playground(
                    &self.selected_model, self.temperature, self.max_tokens,
                    &self.playground_results, self.executing, &self.i18n,
                ).map(Message::Playground)
            }
            RightPanel::Settings => {
                views::settings::view_settings(&self.config, &session.display_name, &self.i18n).map(Message::Settings)
            }
        };

        let right_panel = column![right_tabs, right_content].spacing(4).width(Length::Fixed(360.0));

        // Main layout
        let main = row![left_panel, center_panel, right_panel].spacing(1).height(Length::Fill);

        let content = column![header, horizontal_rule(1), main];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
