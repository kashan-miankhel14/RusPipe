/// M6: Web dashboard + live log streaming via WebSocket.
/// Uses rust-embed to serve static HTML and axum's WebSocket support.
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::info;

use crate::server::AppState;

/// Embed the `assets/` directory at compile time.
#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct Assets;

/// WS /ws/runs/:id/logs — streams live log lines to the browser.
pub async fn ws_logs(
    ws: WebSocketUpgrade,
    Path(run_id): Path<u64>,
    State(state): State<Arc<AppState>>,
) -> Response {
    info!(run_id = run_id, "WebSocket log stream opened");
    ws.on_upgrade(move |socket| handle_ws(socket, run_id, state))
}

async fn handle_ws(mut socket: WebSocket, run_id: u64, state: Arc<AppState>) {
    // Build the first message — drop the lock before any await
    let first_msg = {
        let runs = state.runs.lock().unwrap();
        runs.iter().find(|r| r.id == run_id).map(|run| {
            serde_json::json!({
                "type": "status",
                "run_id": run_id,
                "status": run.status,
                "branch": run.branch,
                "commit": run.commit,
            })
            .to_string()
        })
    }; // MutexGuard dropped here

    match first_msg {
        Some(msg) => {
            if socket.send(Message::Text(msg.into())).await.is_err() {
                return;
            }
        }
        None => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"type": "error", "message": "run not found"})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    }

    // Stream simulated log lines (real impl would tail a broadcast channel)
    let mut ticker = interval(Duration::from_millis(500));
    let mut line = 0u32;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                line += 1;
                let log_msg = serde_json::json!({
                    "type": "log",
                    "run_id": run_id,
                    "line": line,
                    "text": format!("[run {}] step output line {}", run_id, line),
                });
                if socket.send(Message::Text(log_msg.to_string().into())).await.is_err() {
                    break;
                }
                if line >= 20 {
                    let done = serde_json::json!({"type": "done", "run_id": run_id});
                    let _ = socket.send(Message::Text(done.to_string().into())).await;
                    break;
                }
            }
        }
    }

    info!(run_id = run_id, "WebSocket log stream closed");
}
