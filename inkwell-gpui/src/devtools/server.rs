use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

use super::{DevToolsSnapshot, DevToolsCommand};
use super::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use super::handlers;
use super::screenshot;

pub async fn run(
    snapshot: Arc<RwLock<DevToolsSnapshot>>,
    cmd_tx: tokio::sync::mpsc::Sender<DevToolsCommand>,
    start_time: std::time::Instant,
) {
    let sock_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("inkwell");
    let _ = std::fs::create_dir_all(&sock_dir);
    let sock_path = sock_dir.join("devtools.sock");

    // Remove stale socket
    let _ = std::fs::remove_file(&sock_path);

    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => l,
        Err(e) => {
            log::error!("DevTools: failed to bind socket: {}", e);
            return;
        }
    };

    log::info!("DevTools: listening on {}", sock_path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let snap = snapshot.clone();
                let tx = cmd_tx.clone();
                let start = start_time;
                tokio::spawn(async move {
                    handle_connection(stream, snap, tx, start).await;
                });
            }
            Err(e) => {
                log::error!("DevTools: accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    snapshot: Arc<RwLock<DevToolsSnapshot>>,
    cmd_tx: tokio::sync::mpsc::Sender<DevToolsCommand>,
    start_time: std::time::Instant,
) {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                let resp = JsonRpcError::new(serde_json::Value::Null, -32700, "Parse error");
                let _ = writer.write_all(format!("{}\n", resp).as_bytes()).await;
                continue;
            }
        };

        let id = req.id.clone().unwrap_or(serde_json::Value::Null);

        // Dispatch
        let result = dispatch(&req.method, req.params, &snapshot, &cmd_tx, start_time).await;

        let resp = JsonRpcResponse::ok(id, result);
        let _ = writer.write_all(format!("{}\n", resp).as_bytes()).await;
    }
}

async fn dispatch(
    method: &str,
    params: serde_json::Value,
    snapshot: &Arc<RwLock<DevToolsSnapshot>>,
    cmd_tx: &tokio::sync::mpsc::Sender<DevToolsCommand>,
    start_time: std::time::Instant,
) -> serde_json::Value {
    let t0 = std::time::Instant::now();
    let is_read = method.starts_with("devtools/get_")
        || method.starts_with("devtools/list_")
        || matches!(method, "devtools/health_check" | "devtools/app_state" | "devtools/validate_state");
    if !is_read {
        log::info!("[mcp] → {} params={}", method, summarize_params(&params));
    }
    let result = dispatch_inner(method, params, snapshot, cmd_tx, start_time).await;
    let ms = t0.elapsed().as_millis();
    let ok = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(true);
    if !is_read {
        if ok {
            log::info!("[mcp] ← {} ok ({ms}ms)", method);
        } else {
            let err = result.get("error").and_then(|v| v.as_str()).unwrap_or("?");
            log::warn!("[mcp] ← {} FAILED: {err} ({ms}ms)", method);
        }
    }
    result
}

fn summarize_params(params: &serde_json::Value) -> String {
    if params.is_null() || (params.is_object() && params.as_object().unwrap().is_empty()) {
        return "{}".into();
    }
    let s = params.to_string();
    if s.len() > 120 { format!("{}...", &s[..120]) } else { s }
}

async fn dispatch_inner(
    method: &str,
    params: serde_json::Value,
    snapshot: &Arc<RwLock<DevToolsSnapshot>>,
    cmd_tx: &tokio::sync::mpsc::Sender<DevToolsCommand>,
    start_time: std::time::Instant,
) -> serde_json::Value {
    match method {
        // Read handlers (from snapshot, no GPUI round-trip)
        "devtools/health_check" => handlers::health_check(start_time),
        "devtools/app_state" => handlers::app_state(snapshot),
        "devtools/get_project" => handlers::get_project(snapshot),
        "devtools/get_block" => handlers::get_block(snapshot, &params),
        "devtools/get_metrics" => handlers::get_metrics(snapshot),
        "devtools/list_tabs" => handlers::list_tabs(snapshot),
        "devtools/get_logs" => handlers::get_logs(&params),
        "devtools/validate_state" => handlers::validate_state(snapshot),
        "devtools/get_variables" => handlers::get_variables(snapshot),
        "devtools/get_chat_messages" => handlers::get_chat_messages(snapshot, &params),
        "devtools/get_executions" => handlers::get_executions(snapshot, &params),
        "devtools/get_playground_response" => handlers::get_playground_response(snapshot),
        "devtools/get_settings" => handlers::get_settings(snapshot),
        "devtools/list_frameworks" => handlers::list_frameworks(),
        "devtools/list_projects" => handlers::list_projects(),

        // Screenshot
        "devtools/screenshot" => screenshot::capture().await,

        // Write/Action handlers (round-trip to GPUI thread)
        "devtools/set_block" | "devtools/add_block" | "devtools/delete_block"
        | "devtools/toggle_block" | "devtools/reorder_blocks" | "devtools/select_tab"
        | "devtools/select_left_tab" | "devtools/toggle_panel" | "devtools/set_model"
        | "devtools/open_project" | "devtools/new_project" | "devtools/rename_project"
        | "devtools/delete_project"
        | "devtools/set_variable" | "devtools/delete_variable"
        | "devtools/set_dark_mode" | "devtools/set_lang" | "devtools/set_api_key"
        | "devtools/set_github_repo" | "devtools/save_framework" | "devtools/delete_framework"
        | "devtools/create_version" | "devtools/list_versions" | "devtools/restore_version"
        | "devtools/run_prompt" | "devtools/run_sdd" | "devtools/send_chat"
        | "devtools/save_project" => {
            send_command(method, params, cmd_tx).await
        }

        _ => serde_json::json!({"error": format!("Unknown method: {}", method)}),
    }
}

async fn send_command(
    method: &str,
    params: serde_json::Value,
    cmd_tx: &tokio::sync::mpsc::Sender<DevToolsCommand>,
) -> serde_json::Value {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    let cmd = DevToolsCommand {
        method: method.to_string(),
        params,
        response_tx: resp_tx,
    };

    if cmd_tx.send(cmd).await.is_err() {
        return serde_json::json!({"error": "App not responding"});
    }

    match tokio::time::timeout(std::time::Duration::from_secs(5), resp_rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => serde_json::json!({"error": "Command cancelled"}),
        Err(_) => serde_json::json!({"error": "Command timed out (5s)"}),
    }
}
