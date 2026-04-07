use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use bollard::{
    container::{
        Config, CreateContainerOptions, StartContainerOptions,
        AttachContainerOptions, RemoveContainerOptions,
    },
    exec::{CreateExecOptions, StartExecOptions},
    Docker,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::server::AppState;

const SANDBOX_IMAGE: &str = "inkwell-sandbox:latest";

#[derive(Debug, Deserialize)]
pub struct TerminalQuery {
    #[serde(rename = "type")]
    pub session_type: Option<String>,
    // SSH params (used when type=ssh, executed INSIDE the container)
    pub host: Option<String>,
    pub port: Option<String>,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResizeMsg {
    #[serde(rename = "type")]
    msg_type: String,
    cols: u16,
    rows: u16,
}

pub async fn ws_terminal(
    ws: WebSocketUpgrade,
    Query(query): Query<TerminalQuery>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_terminal(socket, query))
}

async fn handle_terminal(socket: WebSocket, query: TerminalQuery) {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Docker connection failed: {e}");
            let (mut tx, _) = socket.split();
            let _ = tx.send(Message::Text(
                format!("\x1b[31mDocker not available: {e}\x1b[0m\r\n").into()
            )).await;
            return;
        }
    };

    let session_type = query.session_type.as_deref().unwrap_or("local");

    // Build the command to run inside the container
    let cmd = match session_type {
        "ssh" => build_ssh_cmd(&query),
        _ => vec!["/bin/zsh".to_string()],
    };

    handle_docker_session(socket, &docker, cmd).await;
}

fn build_ssh_cmd(query: &TerminalQuery) -> Vec<String> {
    let host = query.host.as_deref().unwrap_or("localhost");
    let port = query.port.as_deref().unwrap_or("22");
    let username = query.username.as_deref().unwrap_or("root");

    let mut cmd = vec![
        "ssh".to_string(),
        "-o".to_string(), "StrictHostKeyChecking=accept-new".to_string(),
        "-p".to_string(), port.to_string(),
    ];

    if let Some(key_path) = &query.key_path {
        if query.auth_method.as_deref() == Some("key") {
            cmd.push("-i".to_string());
            cmd.push(key_path.clone());
        }
    }

    cmd.push(format!("{username}@{host}"));
    cmd
}

async fn handle_docker_session(socket: WebSocket, docker: &Docker, cmd: Vec<String>) {
    // Create a unique container name
    let container_name = format!("inkwell-term-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Ensure the sandbox image exists
    if docker.inspect_image(SANDBOX_IMAGE).await.is_err() {
        // Fallback to ubuntu if sandbox image not built yet
        eprintln!("Sandbox image '{SANDBOX_IMAGE}' not found, falling back to ubuntu:22.04");
        if docker.inspect_image("ubuntu:22.04").await.is_err() {
            let (mut tx, _) = socket.split();
            let _ = tx.send(Message::Text(
                "\x1b[31mNo Docker image available. Build the sandbox image first:\r\n  docker build -t inkwell-sandbox -f Dockerfile.sandbox .\x1b[0m\r\n".into()
            )).await;
            return;
        }
    }

    let image = if docker.inspect_image(SANDBOX_IMAGE).await.is_ok() {
        SANDBOX_IMAGE
    } else {
        "ubuntu:22.04"
    };

    // Create container
    let create_result = docker.create_container(
        Some(CreateContainerOptions { name: &container_name, platform: None }),
        Config {
            image: Some(image.to_string()),
            tty: Some(true),
            open_stdin: Some(true),
            cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
            env: Some(vec![
                "TERM=xterm-256color".to_string(),
                "COLORTERM=truecolor".to_string(),
                "LANG=en_US.UTF-8".to_string(),
            ]),
            host_config: Some(bollard::models::HostConfig {
                // Limit container resources
                memory: Some(2 * 1024 * 1024 * 1024), // 2GB RAM
                nano_cpus: Some(2_000_000_000),        // 2 CPU cores
                // Mount SSH keys volume (shared across sessions)
                binds: Some(vec![
                    "inkwell-ssh-keys:/home/devuser/.ssh".to_string(),
                    "inkwell-workspace:/home/devuser/workspace".to_string(),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        },
    ).await;

    let container_id = match create_result {
        Ok(resp) => resp.id,
        Err(e) => {
            eprintln!("Failed to create container: {e}");
            let (mut tx, _) = socket.split();
            let _ = tx.send(Message::Text(
                format!("\x1b[31mFailed to create container: {e}\x1b[0m\r\n").into()
            )).await;
            return;
        }
    };

    // Start container
    if let Err(e) = docker.start_container(&container_id, None::<StartContainerOptions<String>>).await {
        eprintln!("Failed to start container: {e}");
        let _ = docker.remove_container(&container_id, None::<RemoveContainerOptions>).await;
        let (mut tx, _) = socket.split();
        let _ = tx.send(Message::Text(
            format!("\x1b[31mFailed to start container: {e}\x1b[0m\r\n").into()
        )).await;
        return;
    }

    // Attach to the container (stdin + stdout + stderr)
    let attach_result = docker.attach_container(
        &container_id,
        Some(AttachContainerOptions::<String> {
            stdin: Some(true),
            stdout: Some(true),
            stderr: Some(true),
            stream: Some(true),
            ..Default::default()
        }),
    ).await;

    let attach = match attach_result {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to attach to container: {e}");
            let _ = docker.remove_container(&container_id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await;
            return;
        }
    };

    let mut docker_output = attach.output;
    let mut docker_input = attach.input;

    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Docker output -> WebSocket
    let sender_handle = tokio::spawn(async move {
        while let Some(Ok(output)) = docker_output.next().await {
            let bytes = output.into_bytes();
            if ws_sender.send(Message::Binary(bytes.into())).await.is_err() {
                break;
            }
        }
    });

    // WebSocket -> Docker input
    let docker_ref = docker.clone();
    let container_id_clone = container_id.clone();
    let receiver_handle = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(ref text) => {
                    // Check for resize
                    if let Ok(resize) = serde_json::from_str::<ResizeMsg>(text) {
                        if resize.msg_type == "resize" {
                            let _ = docker_ref.resize_container_tty(
                                &container_id_clone,
                                bollard::container::ResizeContainerTtyOptions {
                                    height: resize.rows,
                                    width: resize.cols,
                                },
                            ).await;
                            continue;
                        }
                    }
                    // Terminal input
                    let _ = docker_input.write_all(text.as_bytes()).await;
                }
                Message::Binary(ref data) => {
                    let _ = docker_input.write_all(data).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let _ = tokio::join!(sender_handle, receiver_handle);

    // Cleanup: stop and remove container
    let _ = docker.stop_container(&container_id, None).await;
    let _ = docker.remove_container(
        &container_id,
        Some(RemoveContainerOptions { force: true, ..Default::default() }),
    ).await;
}
