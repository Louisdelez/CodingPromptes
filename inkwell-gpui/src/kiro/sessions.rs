//! Chat session persistence — save/restore conversations.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub name: String,
    pub messages: Vec<(String, String)>, // (role, content)
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default)]
pub struct SessionManager {
    pub sessions: Vec<ChatSession>,
    pub active_session_id: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        let mut mgr = Self::default();
        mgr.new_session("Session 1");
        mgr
    }

    pub fn new_session(&mut self, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        self.sessions.push(ChatSession {
            id: id.clone(), name: name.to_string(),
            messages: vec![], created_at: now, updated_at: now,
        });
        self.active_session_id = Some(id.clone());
        id
    }

    pub fn active_session(&self) -> Option<&ChatSession> {
        self.active_session_id.as_ref()
            .and_then(|id| self.sessions.iter().find(|s| s.id == *id))
    }

    pub fn active_session_mut(&mut self) -> Option<&mut ChatSession> {
        let id = self.active_session_id.clone()?;
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        if let Some(session) = self.active_session_mut() {
            session.messages.push((role.to_string(), content.to_string()));
            session.updated_at = chrono::Utc::now().timestamp_millis();
        }
    }

    pub fn switch_session(&mut self, id: &str) {
        if self.sessions.iter().any(|s| s.id == id) {
            self.active_session_id = Some(id.to_string());
        }
    }

    pub fn delete_session(&mut self, id: &str) {
        self.sessions.retain(|s| s.id != id);
        if self.active_session_id.as_deref() == Some(id) {
            self.active_session_id = self.sessions.first().map(|s| s.id.clone());
        }
    }

    pub fn save(&self, path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.sessions) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                let sessions: Vec<ChatSession> = serde_json::from_str(&json).unwrap_or_default();
                let active = sessions.first().map(|s| s.id.clone());
                Self { sessions, active_session_id: active }
            }
            Err(_) => Self::new(),
        }
    }
}
