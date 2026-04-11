pub mod protocol;
pub mod server;
pub mod handlers;
pub mod mutators;
pub mod actions;
pub mod screenshot;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock, Mutex};
use serde::Serialize;

/// Snapshot of app state, updated every 100ms from the periodic sync.
/// Read handlers serve this directly — no GPUI thread round-trip.
#[derive(Clone, Serialize, Default)]
pub struct DevToolsSnapshot {
    pub screen: String,
    pub project_id: String,
    pub project_name: String,
    pub blocks: Vec<BlockSnapshot>,
    pub projects: Vec<ProjectSummarySnapshot>,
    pub selected_model: String,
    pub cached_prompt: String,
    pub cached_tokens: usize,
    pub cached_chars: usize,
    pub cached_words: usize,
    pub cached_lines: usize,
    pub left_tab: String,
    pub right_tab: String,
    pub left_open: bool,
    pub right_open: bool,
    pub terminal_open: bool,
    pub playground_response: String,
    pub playground_loading: bool,
    pub sdd_running: bool,
    pub dark_mode: bool,
    pub save_status: String,
    pub chat_messages_count: usize,
    pub executions_count: usize,
    pub blocks_enabled: usize,
}

#[derive(Clone, Serialize, Default)]
pub struct BlockSnapshot {
    pub index: usize,
    pub id: String,
    pub block_type: String,
    pub content: String,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Default)]
pub struct ProjectSummarySnapshot {
    pub id: String,
    pub name: String,
}

/// Command sent from the socket server to the GPUI main thread.
pub struct DevToolsCommand {
    pub method: String,
    pub params: serde_json::Value,
    pub response_tx: tokio::sync::oneshot::Sender<serde_json::Value>,
}

/// Shared state between the GPUI app and the socket server.
pub struct DevToolsServer {
    pub snapshot: Arc<RwLock<DevToolsSnapshot>>,
    pub cmd_tx: tokio::sync::mpsc::Sender<DevToolsCommand>,
    pub cmd_rx: tokio::sync::mpsc::Receiver<DevToolsCommand>,
    pub start_time: std::time::Instant,
}

impl DevToolsServer {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(64);
        Self {
            snapshot: Arc::new(RwLock::new(DevToolsSnapshot::default())),
            cmd_tx,
            cmd_rx,
            start_time: std::time::Instant::now(),
        }
    }
}

// Global log ring buffer
static LOG_BUFFER: std::sync::LazyLock<Mutex<VecDeque<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(VecDeque::with_capacity(1000)));

pub fn push_log(msg: String) {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        if buf.len() >= 1000 { buf.pop_front(); }
        buf.push_back(msg);
    }
}

pub fn get_logs(lines: usize) -> Vec<String> {
    if let Ok(buf) = LOG_BUFFER.lock() {
        buf.iter().rev().take(lines).rev().cloned().collect()
    } else {
        vec![]
    }
}
