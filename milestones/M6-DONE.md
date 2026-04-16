# Milestone 6 — Web Dashboard + Live Log Streaming ✅

## Goal
Browser UI that shows live pipeline run status and streams logs in real time via WebSocket.

## Status: DONE

## Files Created
- `src/server/dashboard.rs` — WebSocket handler + rust-embed Assets
- `assets/index.html` — embedded dashboard UI (dark theme, live log viewer)

## Files Modified
- `src/server/mod.rs` — added /dashboard and /ws/runs/:id/logs routes, tracing init
- `Cargo.toml` — added tracing, tracing-subscriber (json), rust-embed, tokio-tungstenite

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `tracing` macros | `mod.rs`, `dashboard.rs` — info!, warn! replace println! |
| `tracing-subscriber` JSON | `mod.rs` — structured JSON log output |
| `rust-embed` | `dashboard.rs` — Assets embeds `assets/` at compile time |
| WebSocket (`axum ws`) | `dashboard.rs` — ws_logs upgrades HTTP → WS |
| `tokio::select!` | `dashboard.rs` — ticker race in log stream loop |
| `Arc<AppState>` | `dashboard.rs` — shared state across WS handler |
| MutexGuard lifetime | `dashboard.rs` — guard dropped before await (Send safety) |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Dashboard visualization | /dashboard serves embedded HTML UI |
| Live log streaming | WS /ws/runs/:id/logs pushes log lines to browser |
| Structured logging | JSON tracing output for log aggregation |
| Distributed tracing | Each request gets a tracing span via TraceLayer |

## Endpoints
```
GET  /dashboard          → serves embedded HTML dashboard
WS   /ws/runs/:id/logs   → streams live log lines as JSON messages
```

## Message Protocol
```json
{"type": "status", "run_id": 1, "status": "triggered", "branch": "main", "commit": "abc123"}
{"type": "log",    "run_id": 1, "line": 1, "text": "[run 1] step output line 1"}
{"type": "done",   "run_id": 1}
{"type": "error",  "message": "run not found"}
```
