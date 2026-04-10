//! Async message handling, state sync, and persistence.
//! Extracted from app/mod.rs to reduce file size.

#![allow(unused_imports)]
use gpui::*;
use gpui_component::input::InputState;
use crate::state::*;
use crate::ui::colors::*;

use super::{InkwellApp, rt};

impl InkwellApp {
    pub(crate) fn poll_messages(&mut self, cx: &mut Context<Self>) {
        // Limit messages per frame to avoid blocking render
        let mut count = 0;
        while count < 50 {
            let msg = match self.state.msg_rx.try_recv() {
                Ok(m) => m,
                Err(_) => break,
            };
            count += 1;
            match msg {
                AsyncMsg::AuthSuccess { session, projects, workspaces } => {
                    self.state.auth_loading = false;
                    crate::persistence::save_session(&crate::persistence::SavedSession {
                        server_url: self.state.server_url.clone(),
                        token: session.token.clone(),
                        email: session.email.clone(),
                        dark_mode: self.state.dark_mode,
                        lang: self.state.lang.clone(),
                        last_project_id: None,
                        left_open: self.state.left_open,
                        right_open: self.state.right_open,
                    });
                    self.state.session = Some(session);
                    self.state.screen = Screen::Ide;
                    // MERGE server projects with local — never overwrite local work
                    for sp in &projects {
                        if !self.state.projects.iter().any(|p| p.id == sp.id) {
                            // New project from server — save locally + add to list
                            let local = crate::persistence::LocalProject {
                                id: sp.id.clone(), name: sp.name.clone(),
                                workspace_id: None,
                                blocks: sp.blocks.clone(),
                                variables: std::collections::HashMap::new(),
                                tags: vec![], framework: sp.framework.clone(),
                                updated_at: chrono::Utc::now().timestamp_millis(),
                            };
                            crate::persistence::save_project(&local);
                            self.state.projects.push(ProjectSummary { id: sp.id.clone(), name: sp.name.clone(), workspace_id: None });
                        }
                    }
                    // Merge workspaces
                    for sw in &workspaces {
                        if !self.state.workspaces.iter().any(|w| w.id == sw.id) {
                            self.state.workspaces.push(sw.clone());
                        }
                    }
                    // Push local projects to server that server doesn't have
                    let local_projects = crate::persistence::load_all_projects();
                    let server_url = self.state.server_url.clone();
                    let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                    for lp in &local_projects {
                        if !projects.iter().any(|sp| sp.id == lp.id) {
                            crate::persistence::sync_project_to_server(&server_url, &token, lp);
                        }
                    }
                    // Don't touch current project — user keeps working on what they had open
                }
                AsyncMsg::AuthError(e) => {
                    self.state.auth_loading = false;
                    self.state.auth_error = Some(e);
                }
                AsyncMsg::LlmResponse(text) => {
                    if text.starts_with("__CHAT__") {
                        self.state.chat_messages.push(("assistant".into(), text[8..].to_string()));
                    } else if text.starts_with("__LOADPROJECT__") {
                        let json_str = &text[15..];
                        if let Ok(proj) = serde_json::from_str::<inkwell_core::types::PromptProject>(json_str) {
                            self.state.project.name = proj.name.clone();
                            self.state.project.id = proj.id.clone();
                            self.state.project.framework = proj.framework.clone();
                            self.state.project.blocks = proj.blocks.iter().map(|b| {
                                Block { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false }
                            }).collect();
                            self.state.block_inputs.clear();
                            self.state.variable_inputs.clear();
                        }
                    } else if text.starts_with("__IMPORT__") {
                        let json_str = &text[10..];
                        if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(json_str) {
                            self.state.undo_stack.push_back(self.state.project.blocks.clone());
                            self.state.project.blocks = blocks.into_iter().map(|b| {
                                Block { id: b.id, block_type: b.block_type, content: b.content, enabled: b.enabled, editing: false }
                            }).collect();
                            self.state.block_inputs.clear();
                            self.state.playground_response = "Imported successfully!".into();
                        } else {
                            self.state.playground_response = "Invalid JSON format for import".into();
                        }
                    } else {
                        self.state.playground_response = text;
                    }
                }
                AsyncMsg::LlmChunk(text) => {
                    self.state.playground_response = text;
                }
                AsyncMsg::LlmDone => {
                    self.state.playground_loading = false;
                    self.state.sdd_running = false;
                    // Execution already tracked via ExecutionRecorded message (local).
                    // Optionally sync to server in background.
                    if !self.state.playground_response.is_empty() {
                        let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                        if !token.is_empty() {
                            let server = self.state.server_url.clone();
                            let project_id = self.state.project.id.clone();
                            let model = self.state.selected_model.clone();
                            let prompt = self.state.cached_prompt.clone();
                            let response = self.state.playground_response.clone();
                            rt().spawn(async move {
                                let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                client.set_token(token);
                                let _ = client.create_execution(&project_id, &serde_json::json!({
                                    "model": model, "provider": "local", "prompt": prompt,
                                    "response": response, "tokens_in": 0, "tokens_out": 0,
                                    "cost": 0.0, "latency_ms": 0,
                                })).await;
                            });
                        }
                    }
                }
                AsyncMsg::LlmError(e) => {
                    self.state.playground_loading = false;
                    self.state.playground_response = format!("Error: {e}");
                }
                AsyncMsg::SddBlockResult { idx, content } => {
                    if let Some(block) = self.state.project.blocks.get_mut(idx) {
                        block.content = content.clone();
                    }
                    // Reset the input state for this block so it picks up new content
                    if idx < self.state.block_inputs.len() {
                        self.state.block_inputs[idx] = None; // Will be recreated next frame
                    }
                }
                AsyncMsg::ExportReady(path) => {
                    self.state.playground_response = format!("Exported to {path}");
                }
                AsyncMsg::VersionsLoaded(versions) => {
                    self.state.versions = versions;
                }
                AsyncMsg::NodesLoaded(nodes) => {
                    self.state.gpu_nodes = nodes;
                }
                AsyncMsg::SttResult { block_idx, text } => {
                    self.state.stt_recording = false;
                    if let Some(block) = self.state.project.blocks.get_mut(block_idx) {
                        if !block.content.is_empty() && !block.content.ends_with(' ') && !block.content.ends_with('\n') {
                            block.content.push(' ');
                        }
                        block.content.push_str(&text);
                    }
                    // Reset input to pick up new content
                    if block_idx < self.state.block_inputs.len() {
                        self.state.block_inputs[block_idx] = None;
                    }
                }
                AsyncMsg::SttError(e) => {
                    self.state.stt_recording = false;
                    self.state.playground_response = format!("STT Error: {e}");
                }
                AsyncMsg::CustomFrameworkSaved => {}
                AsyncMsg::MultiModelResult { model, response } => {
                    self.state.multi_model_responses.push((model, response));
                }
                AsyncMsg::MultiModelDone => {
                    self.state.multi_model_loading = false;
                }
                AsyncMsg::ExecutionRecorded(exec) => {
                    self.state.executions.push(exec);
                    // Cap at 500 executions to prevent unbounded growth
                    if self.state.executions.len() > 500 {
                        self.state.executions.drain(..self.state.executions.len() - 500);
                    }
                }
                AsyncMsg::CollabUsersLoaded(users) => {
                    self.state.collab_users = users;
                }
                AsyncMsg::GitHubPushed(msg) => {
                    self.state.playground_response = msg;
                }
                AsyncMsg::TerminalOutput(text) => {
                    let idx = self.state.active_terminal;
                    if let Some(session) = self.state.terminal_sessions.get_mut(idx) {
                        session.output.push_str(&text);
                        if session.output.len() > 10_000 {
                            let mut start = session.output.len() - 8_000;
                            // Ensure we don't split a UTF-8 character
                            while start < session.output.len() && !session.output.is_char_boundary(start) {
                                start += 1;
                            }
                            session.output = session.output[start..].to_string();
                        }
                    }
                }
            }
        }
        // Sync key state → store after processing messages
        if count > 0 {
            self.store.update(cx, |s, cx| {
                if s.playground_response != self.state.playground_response {
                    s.playground_response = self.state.playground_response.clone();
                    s.playground_loading = self.state.playground_loading;
                    cx.emit(crate::store::StoreEvent::PlaygroundUpdated);
                }
                if s.save_status != self.state.save_status {
                    s.save_status = self.state.save_status;
                    cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                }
                if s.session.is_some() != self.state.session.is_some() {
                    s.session = self.state.session.clone();
                    s.screen = self.state.screen;
                    cx.emit(crate::store::StoreEvent::SessionChanged);
                }
                // Sync project blocks for SDD/import results
                if s.project.blocks.len() != self.state.project.blocks.len() || self.state.prompt_dirty {
                    s.project.blocks = self.state.project.blocks.iter().map(|b| {
                        Block { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false }
                    }).collect();
                    s.prompt_dirty = true;
                    s.refresh_cache();
                    self.state.prompt_dirty = false;
                    cx.emit(crate::store::StoreEvent::PromptCacheUpdated);
                    cx.emit(crate::store::StoreEvent::ProjectChanged);
                }
            });
        }
    }
}

impl InkwellApp {

    pub(crate) fn sync_block_content(&mut self, cx: &mut Context<Self>) {
        // Read content from Input widgets — only allocate if value changed
        let mut changed = false;
        for (idx, block) in self.state.project.blocks.iter_mut().enumerate() {
            if let Some(Some(input)) = self.state.block_inputs.get(idx) {
                let val = input.read(cx).value();
                if val != block.content.as_str() {
                    block.content = val.to_string();
                    changed = true;
                }
            }
        }
        // Read variable values — only allocate if changed
        let var_keys: Vec<String> = self.state.variable_inputs.keys().cloned().collect();
        for var_name in var_keys {
            if let Some(entity) = self.state.variable_inputs.get(&var_name) {
                let val = entity.read(cx).value();
                let old = self.state.project.variables.get(&var_name).map(|s| s.as_str()).unwrap_or("");
                if val != old && !val.is_empty() {
                    self.state.project.variables.insert(var_name, val.to_string());
                    changed = true;
                }
            }
        }
        // Refresh prompt cache if dirty
        if changed { self.state.prompt_dirty = true; }
        if self.state.prompt_dirty {
            let core_blocks: Vec<inkwell_core::types::PromptBlock> = self.state.project.blocks.iter().map(|b| {
                inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
            }).collect();
            self.state.cached_prompt = inkwell_core::prompt::compile_prompt(&core_blocks, &self.state.project.variables);
            self.state.cached_tokens = (self.state.cached_prompt.len() as f64 / 4.0).ceil() as usize;
            self.state.cached_chars = self.state.cached_prompt.len();
            self.state.cached_words = if self.state.cached_prompt.is_empty() { 0 } else { self.state.cached_prompt.split_whitespace().count() };
            self.state.cached_lines = self.state.cached_prompt.lines().count();
            self.state.cached_vars = inkwell_core::prompt::extract_variables(&core_blocks);
            self.state.prompt_dirty = false;
        }
        // Read search query from input (only allocate if changed)
        if let Some(ref input) = self.state.search_input {
            let val = input.read(cx).value();
            if val != self.state.search_query.as_str() {
                self.state.search_query = val.to_string();
            }
        }
        // Mark save pending if content changed (actual save in periodic timer)
        if changed {
            self.state.save_pending = true;
        }
    }

    pub(crate) fn save_to_backend(&mut self) {
        self.state.save_status = "saving";

        // 1. Save locally FIRST (instant, no network)
        let local_project = crate::persistence::LocalProject {
            id: self.state.project.id.clone(),
            name: self.state.project.name.clone(),
            workspace_id: self.state.project.workspace_id.clone(),
            blocks: self.state.project.blocks.iter().map(|b| {
                inkwell_core::types::PromptBlock {
                    id: b.id.clone(), block_type: b.block_type,
                    content: b.content.clone(), enabled: b.enabled,
                }
            }).collect(),
            variables: self.state.project.variables.clone(),
            tags: self.state.project.tags.clone(),
            framework: self.state.project.framework.clone(),
            updated_at: chrono::Utc::now().timestamp_millis(),
        };
        crate::persistence::save_project(&local_project);

        // Also save custom frameworks locally
        let local_fws: Vec<crate::persistence::LocalFramework> = self.state.custom_frameworks.iter()
            .map(|f| crate::persistence::LocalFramework { name: f.name.clone(), blocks: f.blocks.clone() })
            .collect();
        crate::persistence::save_frameworks(&local_fws);

        // Save settings (both old format + new structured format)
        crate::persistence::save_settings(&crate::persistence::LocalSettings {
            api_key_openai: self.state.api_key_openai.clone(),
            api_key_anthropic: self.state.api_key_anthropic.clone(),
            api_key_google: self.state.api_key_google.clone(),
            github_repo: self.state.github_repo.clone(),
            selected_model: self.state.selected_model.clone(),
        });
        crate::settings::AppSettings {
            theme: if self.state.dark_mode { "dark".into() } else { "light".into() },
            lang: self.state.lang.clone(),
            server_url: self.state.server_url.clone(),
            api_keys: crate::settings::ApiKeys {
                openai: self.state.api_key_openai.clone(),
                anthropic: self.state.api_key_anthropic.clone(),
                google: self.state.api_key_google.clone(),
            },
            github_repo: self.state.github_repo.clone(),
            selected_model: self.state.selected_model.clone(),
        }.save();

        // Save layout state
        crate::layout::SavedLayout {
            left_open: self.state.left_open,
            left_width: self.state.left_open as u32 as f32 * 288.0, // TODO: read from store
            right_open: self.state.right_open,
            right_width: 384.0,
            terminal_open: false,
            dark_mode: self.state.dark_mode,
        }.save();

        // 2. Background sync to server (non-blocking, best-effort)
        let server_url = self.state.server_url.clone();
        let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
        crate::persistence::sync_project_to_server(&server_url, &token, &local_project);
    }

    pub(crate) fn ensure_terminal_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.terminal_input_entity.is_none() {
            self.state.terminal_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Enter command...")
            }));
        }
        if self.state.chat_input_entity.is_none() {
            self.state.chat_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Type a message...")
            }));
        }
        if self.state.ssh_host_input.is_none() {
            self.state.ssh_host_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("hostname or IP")
            }));
        }
        if self.state.ssh_user_input.is_none() {
            self.state.ssh_user_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("username")
            }));
        }
        if self.state.tag_input.is_none() {
            self.state.tag_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("tag name")
            }));
        }
        if self.state.version_label_input.is_none() {
            self.state.version_label_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("version label")
            }));
        }
        if self.settings_inputs.openai.is_none() {
            self.settings_inputs.openai = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("sk-...").masked(true)
            }));
        }
        if self.settings_inputs.anthropic.is_none() {
            self.settings_inputs.anthropic = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("sk-ant-...").masked(true)
            }));
        }
        if self.settings_inputs.google.is_none() {
            self.settings_inputs.google = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("AIza...").masked(true)
            }));
        }
        if self.settings_inputs.ssh_port.is_none() {
            self.settings_inputs.ssh_port = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value("22")
            }));
        }
        if self.state.workspace_name_input.is_none() && self.state.editing_workspace_id.is_some() {
            let name = self.state.workspaces.iter()
                .find(|w| Some(w.id.as_str()) == self.state.editing_workspace_id.as_deref())
                .map(|w| w.name.clone()).unwrap_or_default();
            self.state.workspace_name_input = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value(name)
            }));
        }
        if self.state.search_input.is_none() {
            self.state.search_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Rechercher...")
            }));
        }
        if self.state.name_input_entity.is_none() {
            let name = self.state.project.name.clone();
            self.state.name_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value(name)
            }));
        }
        if self.state.framework_name_input.is_none() {
            self.state.framework_name_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Framework name")
            }));
        }
        if self.settings_inputs.github_repo.is_none() {
            self.settings_inputs.github_repo = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("owner/repo")
            }));
        }
    }

    pub(crate) fn ensure_block_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Ensure we have an InputState for each block
        while self.state.block_inputs.len() < self.state.project.blocks.len() {
            let idx = self.state.block_inputs.len();
            let content = self.state.project.blocks.get(idx)
                .map(|b| b.content.clone()).unwrap_or_default();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(content)
                    .multi_line(true)
                    .auto_grow(3, 20)
            });
            self.state.block_inputs.push(Some(input));
        }
        // Remove excess
        self.state.block_inputs.truncate(self.state.project.blocks.len());

        // Ensure variable input entities — only when vars changed
        let var_count = self.state.cached_vars.len();
        if var_count != self.state.variable_inputs.len() || self.state.variable_inputs.keys().any(|k| !self.state.cached_vars.contains(k)) {
            for var in &self.state.cached_vars.clone() {
                if !self.state.variable_inputs.contains_key(var) {
                    let val = self.state.project.variables.get(var).cloned().unwrap_or_default();
                    let entity = cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder(format!("value for {var}"))
                            .default_value(val)
                    });
                    self.state.variable_inputs.insert(var.clone(), entity);
                }
            }
            let cached = self.state.cached_vars.clone();
            self.state.variable_inputs.retain(|k, _| cached.contains(k));
        }
    }
}
