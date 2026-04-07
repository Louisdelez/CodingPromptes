use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
use tauri::Emitter;

struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Mutex<Box<dyn Write + Send>>,
}

pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn spawn_local(
        &mut self,
        app: &tauri::AppHandle,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        if let Ok(home) = std::env::var("HOME") {
            cmd.cwd(&home);
        }
        self.spawn_session(app, session_id, cmd, cols, rows)
    }

    pub fn spawn_ssh(
        &mut self,
        app: &tauri::AppHandle,
        session_id: &str,
        host: &str,
        port: u16,
        username: &str,
        auth_method: &str,
        password: Option<&str>,
        key_path: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let _ = password; // SSH password auth handled by the ssh process prompting

        let mut cmd = CommandBuilder::new("ssh");
        cmd.arg("-o");
        cmd.arg("StrictHostKeyChecking=accept-new");
        cmd.arg("-p");
        cmd.arg(port.to_string());

        if auth_method == "key" {
            if let Some(key) = key_path {
                cmd.arg("-i");
                cmd.arg(key);
            }
        }

        cmd.arg(format!("{username}@{host}"));
        cmd.env("TERM", "xterm-256color");

        self.spawn_session(app, session_id, cmd, cols, rows)
    }

    fn spawn_session(
        &mut self,
        app: &tauri::AppHandle,
        session_id: &str,
        cmd: CommandBuilder,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let pty_system = NativePtySystem::default();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {e}"))?;

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn: {e}"))?;

        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();

        // Spawn reader thread -> emit events to frontend
        let event_name = format!("pty-output-{session_id}");
        let app_handle = app.clone();

        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_handle.emit(&event_name, text);
                    }
                    Err(_) => break,
                }
            }
        });

        self.sessions.insert(
            session_id.to_string(),
            PtySession {
                master: pair.master,
                writer: Mutex::new(writer),
            },
        );

        Ok(())
    }

    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        if let Some(session) = self.sessions.get(session_id) {
            let mut writer = session.writer.lock().map_err(|e| e.to_string())?;
            writer.write_all(data).map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        if let Some(session) = self.sessions.get(session_id) {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| e.to_string())
        } else {
            Err("Session not found".into())
        }
    }

    pub fn kill(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }
}
