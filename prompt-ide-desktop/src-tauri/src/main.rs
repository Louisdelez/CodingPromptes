#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod pty_manager;

use pty_manager::PtyManager;
use std::sync::Mutex;
use tauri::State;

struct AppPtyManager(Mutex<PtyManager>);

#[tauri::command]
fn spawn_pty(
    state: State<'_, AppPtyManager>,
    app: tauri::AppHandle,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.spawn_local(&app, &session_id, cols, rows)
}

#[tauri::command]
fn spawn_ssh(
    state: State<'_, AppPtyManager>,
    app: tauri::AppHandle,
    session_id: String,
    host: String,
    port: u16,
    username: String,
    auth_method: String,
    password: Option<String>,
    key_path: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.spawn_ssh(
        &app,
        &session_id,
        &host,
        port,
        &username,
        &auth_method,
        password.as_deref(),
        key_path.as_deref(),
        cols,
        rows,
    )
}

#[tauri::command]
fn write_pty(state: State<'_, AppPtyManager>, session_id: String, data: String) -> Result<(), String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.write(&session_id, data.as_bytes())
}

#[tauri::command]
fn resize_pty(
    state: State<'_, AppPtyManager>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.resize(&session_id, cols, rows)
}

#[tauri::command]
fn kill_pty(state: State<'_, AppPtyManager>, session_id: String) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| e.to_string())?;
    mgr.kill(&session_id);
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppPtyManager(Mutex::new(PtyManager::new())))
        .invoke_handler(tauri::generate_handler![
            spawn_pty,
            spawn_ssh,
            write_pty,
            resize_pty,
            kill_pty,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
