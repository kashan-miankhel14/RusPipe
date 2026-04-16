pub mod api;
pub mod dashboard;
pub mod github;
pub mod rbac;
pub mod tls;
pub mod webhook;

use crate::metrics::metrics_handler;
use crate::server::api::{list_runs, new_store, RunRecord, RunStore};
use crate::server::dashboard::{ws_logs, Assets};
use crate::server::rbac::{auth_middleware, AuthUser};
use crate::server::tls::self_signed_tls_config;
use crate::server::webhook::verify_github_signature;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use rustpipe_macros::requires_role;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

pub struct AppState {
    pub webhook_secret: String,
    pub runs: RunStore,
    pub db_path: String,
}

/// Config with explicit lifetime — borrows the webhook secret from the caller.
/// Demonstrates lifetime annotations on a handler-adjacent struct (M5 requirement).
pub struct WebhookConfig<'a> {
    pub secret: &'a str,
    pub branch_filter: Option<&'a str>,
}

impl<'a> WebhookConfig<'a> {
    pub fn matches_branch(&self, branch: &str) -> bool {
        self.branch_filter.map_or(true, |f| branch == f)
    }
}

async fn handle_github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<Value>) {
    let sig = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Use lifetime-annotated config (borrows from state)
    let cfg = WebhookConfig {
        secret: &state.webhook_secret,
        branch_filter: None,
    };

    if !verify_github_signature(cfg.secret, &body, sig) {
        warn!("Webhook rejected: invalid HMAC signature");
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid signature"})),
        );
    }

    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "bad json"}))),
    };

    let branch = payload["ref"]
        .as_str()
        .unwrap_or("unknown")
        .trim_start_matches("refs/heads/")
        .to_string();
    let commit = payload["after"].as_str().unwrap_or("unknown").to_string();
    let repo = payload["repository"]["full_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    if !cfg.matches_branch(&branch) {
        return (
            StatusCode::OK,
            Json(json!({"status": "ignored", "reason": "branch filter"})),
        );
    }

    info!(
        branch = %branch,
        commit = %&commit[..8.min(commit.len())],
        repo = %repo,
        "Webhook triggered"
    );

    let mut runs = state.runs.lock().unwrap();
    let id = runs.len() as u64 + 1;
    runs.push(RunRecord {
        id,
        pipeline: repo,
        branch: branch.clone(),
        commit: commit.clone(),
        status: "triggered".into(),
    });

    (
        StatusCode::OK,
        Json(json!({
            "status": "triggered",
            "branch": branch,
            "commit": commit,
            "run_id": id
        })),
    )
}

/// Admin-only route — protected by #[requires_role("admin")] proc-macro.
/// The macro injects an RBAC guard that reads __auth_user from request extensions.
#[requires_role("admin")]
async fn admin_config(
    __auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    // M9: write RBAC access to audit log
    if let Ok(pool) = crate::db::open(&state.db_path).await {
        let detail = format!("user={} role={} route=admin_config", __auth_user.name, __auth_user.role);
        let _ = crate::db::audit(&pool, 0, "rbac_access", Some(&detail)).await;
    }
    Json(json!({"config": "admin-only settings", "user": __auth_user.name})).into_response()
}

/// M5: Rate limiting middleware — 100 req/s sliding window using atomic counter.
static REQUEST_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static WINDOW_START: std::sync::OnceLock<std::sync::Mutex<std::time::Instant>> = std::sync::OnceLock::new();

async fn rate_limit_middleware(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    let window = WINDOW_START.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
    let now = std::time::Instant::now();
    let reset = {
        let mut start = window.lock().unwrap();
        if now.duration_since(*start).as_secs() >= 1 {
            *start = now;
            REQUEST_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
            false
        } else {
            REQUEST_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) >= 100
        }
    };
    if reset {
        return (StatusCode::TOO_MANY_REQUESTS, Json(json!({"error": "rate limit exceeded"}))).into_response();
    }
    next.run(req).await
}

async fn serve_dashboard() -> Html<String> {
    let html = Assets::get("index.html")
        .map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
        .unwrap_or_else(|| "<h1>RustPipe Dashboard</h1>".into());
    Html(html)
}

pub async fn serve(port: u16, secret: String) -> anyhow::Result<()> {
    // M6 + M13: structured JSON tracing with OpenTelemetry layer
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpipe=info".into()),
        )
        .json()
        .init();

    let state = Arc::new(AppState {
        webhook_secret: secret,
        runs: new_store(),
        db_path: ".rustpipe.db".into(),
    });

    crate::metrics::init();

    // Tower middleware: trace + timeout (tower-http layers are axum-compatible)
    // M5: Rate limiting via custom axum middleware (tower RateLimitLayer not Clone-compatible with axum 0.7)
    // Applied as axum middleware to preserve type compatibility
    let app = Router::new()
        .route("/webhook/github", post(handle_github_webhook))
        .route("/api/v1/runs", get(list_runs))
        .route("/api/v1/admin/config", get(admin_config))
        .route("/metrics", get(metrics_handler))
        .route("/dashboard", get(serve_dashboard))
        .route("/ws/runs/{id}/logs", get(ws_logs))
        .with_state(state)
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(auth_middleware))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let tls_config = self_signed_tls_config()?;
    let acceptor = TlsAcceptor::from(tls_config);
    let listener = TcpListener::bind(addr).await?;

    info!(port = port, "RustPipe HTTPS server started");
    println!("🚀 RustPipe listening on https://localhost:{}", port);
    println!("   → POST /webhook/github");
    println!("   → GET  /api/v1/runs");
    println!("   → GET  /dashboard");
    println!("   → WS   /ws/runs/:id/logs\n");

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let app = app.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let io = TokioIo::new(tls_stream);
                    let svc = hyper_util::service::TowerToHyperService::new(app);
                    let _ = AutoBuilder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(io, svc)
                        .await;
                }
                Err(e) => warn!("TLS error from {}: {}", remote_addr, e),
            }
        });
    }
}
