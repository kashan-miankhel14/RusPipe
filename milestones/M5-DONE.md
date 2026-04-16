# Milestone 5 — Webhook Server + TLS ✅

## Goal
Trigger pipelines automatically on git push via webhooks, served over HTTPS.

## Status: DONE

## Files Created
- `src/server/mod.rs` — axum HTTPS server, tower middleware, TLS accept loop
- `src/server/api.rs` — RunRecord, RunStore, GET /api/v1/runs
- `src/server/webhook.rs` — HMAC-SHA256 GitHub signature verification
- `src/server/tls.rs` — rcgen self-signed cert + rustls ServerConfig
- `src/server/github.rs` — hyper low-level HTTPS client for GitHub API

## Files Modified
- `src/main.rs` — added `serve` subcommand
- `Cargo.toml` — added axum, tower, tower-http, hyper, hyper-util, hyper-rustls, rustls, tokio-rustls, rcgen, hmac, sha2, hex, http-body-util

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `axum` HTTP server | `mod.rs` — Router with routes and State |
| `tower` middleware | `mod.rs` — TimeoutLayer + TraceLayer stacked |
| `hyper` low-level HTTP | `github.rs` — Client::builder(TokioExecutor) for GitHub API |
| `rustls` TLS | `tls.rs` — ServerConfig with self-signed cert |
| Lifetimes | `mod.rs` — `WebhookConfig<'a>` borrows secret from AppState |
| `hmac` + `sha2` | `webhook.rs` — X-Hub-Signature-256 verification |
| `rcgen` | `tls.rs` — generates self-signed cert at runtime |
| `tokio-rustls` | `mod.rs` — TlsAcceptor wraps TCP stream |
| `hyper-util` | `mod.rs` — TowerToHyperService, AutoBuilder, TokioIo |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| GitOps workflow | Webhook triggers pipeline on git push |
| Webhook-driven automation | POST /webhook/github parses push events |
| HMAC request authentication | X-Hub-Signature-256 verified before processing |
| HTTPS / TLS | Self-signed cert served via rustls |
| Request timeout | 30s timeout via tower-http TimeoutLayer |
| Structured request tracing | TraceLayer logs every request |

## Commands
```bash
rustpipe serve --port 9090 --secret my-webhook-secret

# Test webhook (no real HMAC):
curl -k -X POST https://localhost:9090/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=invalid" \
  -d '{"ref":"refs/heads/main","after":"abc123","repository":{"full_name":"user/repo"}}'

# List runs:
curl -k https://localhost:9090/api/v1/runs
```
