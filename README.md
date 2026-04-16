# RustPipe

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Tests](https://img.shields.io/badge/tests-24%20passing-brightgreen)](#)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](#)

A self-hosted CI/CD pipeline engine written in Rust. Define pipelines in YAML or a custom DSL, run them locally or in Docker, stream live logs over WebSocket, and receive GitHub commit status checks — all from a single binary.

---

## Demo

```
$ rustpipe run --pipeline .rustpipe.yml

⚡ Running pipeline: my-pipeline

┌ Stage: lint [rust:latest]
  ▶ lint › Run clippy
    Checking rustpipe v0.1.0
  ✓ Run clippy
└ Stage lint PASSED

┌ Stage: test [rust:latest]
  ▶ test › Run tests
    running 24 tests ... ok
  ✓ Run tests
└ Stage test PASSED

┌ Stage: build [rust:latest]
  ▶ build › Build release
    Compiling rustpipe v0.1.0
  ✓ Build release
└ Stage build PASSED

✅ Pipeline complete
```

---

## Why RustPipe?

Most CI systems are black boxes you can't run locally. RustPipe is a single binary that:

- Runs your pipeline **locally** with the same logic as your CI server
- Gives you **live log streaming** via WebSocket dashboard
- Posts **GitHub commit status checks** on every run
- Detects **pipeline drift** when your config changes between runs
- Masks **secrets** and warns on hardcoded values before execution

---

## Quick Start

```bash
# Build
cargo build --release

# Scaffold a pipeline in the current directory
./target/release/rustpipe init

# Validate your pipeline
./target/release/rustpipe validate .rustpipe.yml

# Dry run — see what would execute without running anything
./target/release/rustpipe run --dry-run

# Run locally (shell)
./target/release/rustpipe run

# Run with Docker isolation per stage
./target/release/rustpipe run --docker

# Run with DAG-based parallel execution
./target/release/rustpipe run --parallel

# Start HTTPS webhook server + live dashboard
./target/release/rustpipe serve --port 9090 --secret my-webhook-secret

# View run history
./target/release/rustpipe history

# View logs for a specific run
./target/release/rustpipe logs 1

# Clear artifact cache
./target/release/rustpipe cache clear
```

---

## Pipeline Configuration

### YAML (`.rustpipe.yml`)

```yaml
name: my-pipeline

trigger:
  branches: [main, develop]
  events: [push, pull_request]

secrets:
  - TOKEN
  - DB_PASSWORD

notify:
  slack:   https://hooks.slack.com/services/...
  discord: https://discord.com/api/webhooks/...

stages:
  lint:
    runs-on: rust:latest
    steps:
      - name: Run clippy
        run: cargo clippy -- -D warnings

  test:
    runs-on: rust:latest
    needs: [lint]
    when: branch == "main"
    timeout_secs: 120
    steps:
      - name: Run tests
        run: cargo test --all
        retry:
          attempts: 3
          backoff: exponential

  build:
    runs-on: rust:latest
    needs: [test]
    matrix:
      rust: [stable, beta]
      os: [linux, macos]
    fail-fast: true
    steps:
      - name: Build release
        run: cargo build --release
        artifact: app-binary
```

### DSL (`.rustpipe`)

```
pipeline my-pipeline

stage lint
  runs-on rust:latest
  step "Run clippy"
    run cargo clippy -- -D warnings
  end
end

stage test
  runs-on rust:latest
  needs lint
  when branch == "main"
  step "Run tests"
    run cargo test --all
  end
end
```

Both formats are auto-detected by file extension.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                        CLI (clap)                        │
│   validate │ run │ serve │ agent │ history │ logs │ cache │
└──────────────────────────┬──────────────────────────────┘
                           │
          ┌────────────────▼────────────────┐
          │         Pipeline Engine          │
          │  YAML parser  │  DSL (nom)       │
          │  Validator    │  DAG scheduler   │
          │  Typestate    │  Drift detection │
          └────────────────┬────────────────┘
                           │
     ┌─────────────────────┼─────────────────────┐
     │                     │                     │
     ▼                     ▼                     ▼
┌─────────┐         ┌─────────────┐       ┌──────────────┐
│  Shell  │         │   Docker    │       │   Parallel   │
│ Executor│         │  Executor   │       │  (rayon+DAG) │
│ + retry │         │  (bollard)  │       │              │
└─────────┘         └─────────────┘       └──────────────┘
     │
     ├── Matrix builds (fan-out per combination)
     ├── Secret masking + hardcoded detection
     ├── Artifact caching (SHA-256 + memmap2)
     └── Step logging → SQLite

┌─────────────────────────────────────────────────────────┐
│                   HTTPS Server (axum)                    │
│  POST /webhook/github  │  HMAC-SHA256 verification       │
│  GET  /api/v1/runs     │  RBAC (viewer/operator/admin)   │
│  GET  /metrics         │  Prometheus counters + histograms│
│  GET  /dashboard       │  Embedded HTML (rust-embed)     │
│  WS   /ws/runs/:id/logs│  Live log streaming             │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│              Distributed Runners (gRPC + mTLS)           │
│  rustpipe agent --server https://... --id agent-1        │
│  Round-robin dispatch │ Service discovery │ libc FFI     │
└─────────────────────────────────────────────────────────┘
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `rustpipe validate <file>` | Validate pipeline, show dependency graph |
| `rustpipe init` | Scaffold `.rustpipe.yml` in current directory |
| `rustpipe run` | Run pipeline locally (shell) |
| `rustpipe run --docker` | Run stages in Docker containers |
| `rustpipe run --parallel` | DAG-based parallel execution |
| `rustpipe run --cache` | Skip unchanged stages via artifact cache |
| `rustpipe run --dry-run` | Show execution plan without running |
| `rustpipe serve --port 9090` | Start HTTPS webhook + dashboard server |
| `rustpipe agent --server <url>` | Start a remote runner agent |
| `rustpipe history` | List last 10 pipeline runs |
| `rustpipe history --limit 25` | List last N runs |
| `rustpipe logs <run-id>` | Print stored logs for a run |
| `rustpipe cache clear` | Wipe `.rustpipe-cache/` |

---

## Server Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| `POST` | `/webhook/github` | Receive GitHub push events | HMAC-SHA256 |
| `GET` | `/api/v1/runs` | List recent pipeline runs | viewer+ |
| `GET` | `/api/v1/admin/config` | Admin configuration | admin |
| `GET` | `/metrics` | Prometheus metrics | — |
| `GET` | `/dashboard` | Web dashboard UI | — |
| `WS` | `/ws/runs/:id/logs` | Live log streaming | — |

### RBAC

```bash
# Role hierarchy: viewer < operator < admin
curl -k -H "X-User: alice" -H "X-Role: admin"  https://localhost:9090/api/v1/admin/config
curl -k -H "X-User: bob"   -H "X-Role: viewer" https://localhost:9090/api/v1/runs
```

---

## Secrets

Secrets are loaded from environment variables prefixed with `RUSTPIPE_SECRET_`:

```bash
export RUSTPIPE_SECRET_TOKEN=ghp_abc123
export RUSTPIPE_SECRET_DB_PASSWORD=supersecret
rustpipe run
```

- Injected as `SECRET_TOKEN`, `SECRET_DB_PASSWORD` into container stages
- Any secret value appearing literally in a `run:` command triggers a warning
- Secret values are never stored in the run history database

---

## Artifact Caching

```bash
rustpipe run --cache
```

Hashes each stage's step commands using SHA-256 + memmap2. Skips stages whose inputs haven't changed since the last successful run. Clear with `rustpipe cache clear`.

---

## Matrix Builds

```yaml
stages:
  test:
    matrix:
      rust: [stable, beta, nightly]
      os: [linux, macos]
    fail-fast: true
```

Generates 6 parallel jobs. Each receives matrix variables as env vars (`RUST=stable`, `OS=linux`, etc.).

---

## Distributed Runners

```bash
# Server
rustpipe serve --port 9090

# Agents register on connect, receive jobs via gRPC + mTLS
rustpipe agent --server https://localhost:9090 --id agent-1 --labels linux,rust
rustpipe agent --server https://localhost:9090 --id agent-2 --labels gpu,linux
```

Jobs are dispatched round-robin across available agents. All gRPC connections use mutual TLS.

---

## Metrics

```bash
curl -k https://localhost:9090/metrics
```

| Metric | Description |
|--------|-------------|
| `rustpipe_runs_total{status="passed\|failed"}` | Run counter |
| `rustpipe_stage_duration_seconds{stage=...}` | Duration histogram (p50/p95/p99) |

---

## Drift Detection

After each successful run, the pipeline file hash (SHA-256) is stored in `.rustpipe.db`. On the next run, if the file has changed:

```
⚠ Pipeline definition has changed since last successful run (drift detected)
```

---

## Project Structure

```
src/
├── main.rs              # CLI entry point (clap)
├── lib.rs               # Public API surface
├── error.rs             # thiserror error types
├── cli/mod.rs           # CLI command handlers
├── pipeline/
│   ├── model.rs         # Pipeline, Stage, Step structs
│   ├── validator.rs     # Validator trait + impls
│   ├── dag.rs           # petgraph DAG, typestate, Scheduler<T>
│   ├── dsl.rs           # nom DSL parser
│   └── mod.rs           # parse() — YAML + DSL auto-detect
├── runner/
│   ├── shell.rs         # tokio shell executor + retry + tracing
│   ├── container.rs     # bollard Docker executor
│   ├── parallel.rs      # DAG parallel executor (rayon + tokio)
│   └── matrix.rs        # matrix build fan-out
├── server/
│   ├── mod.rs           # axum HTTPS server, tower middleware
│   ├── api.rs           # GET /api/v1/runs
│   ├── webhook.rs       # HMAC-SHA256 signature verification
│   ├── tls.rs           # rcgen self-signed cert + rustls
│   ├── github.rs        # hyper GitHub API client + GitOpsLoop
│   ├── rbac.rs          # AuthUser, role_allows(), auth_middleware
│   └── dashboard.rs     # WebSocket log streaming + rust-embed
├── agent/
│   ├── agent.rs         # gRPC runner agent (tonic + mTLS + unsafe FFI)
│   └── registry.rs      # RunnerRegistry (round-robin dispatch)
├── db/mod.rs            # sqlx SQLite — runs, audit_log, step_logs
├── cache/mod.rs         # SHA-256 + memmap2 artifact caching
├── secrets/mod.rs       # env secret loading, masking, hardcoded detection
├── notify/mod.rs        # Slack/Discord/webhook notifications + retry
└── metrics/mod.rs       # Prometheus metrics + statrs percentiles
rustpipe-macros/
└── src/lib.rs           # #[requires_role("admin")] proc-macro
proto/runner.proto        # gRPC RunnerService definition
tests/integration.rs      # 24 integration tests
migrations/               # SQLite schema migrations
```

---

## Rust Concepts Demonstrated

| Concept | Where |
|---------|-------|
| `serde` + `serde_yaml` | `pipeline/model.rs` |
| `clap` derive macros | `main.rs` |
| `thiserror` / `anyhow` | `error.rs`, `cli/mod.rs` |
| Traits + trait objects | `pipeline/validator.rs` |
| Generics + monomorphization | `pipeline/dag.rs` (`Scheduler<T>`) |
| Typestate pattern | `pipeline/dag.rs` (`TypedPipeline<State>`) |
| `PhantomData` | `pipeline/dag.rs`, `db/mod.rs` |
| `tokio` async + `select!` | `runner/container.rs` |
| `Arc` / `Mutex` / `RwLock` | `runner/container.rs`, `server/mod.rs` |
| `mpsc` + `broadcast` channels | `runner/parallel.rs` |
| `bollard` Docker SDK | `runner/container.rs` |
| `petgraph` DAG | `pipeline/dag.rs` |
| `rayon` parallelism | `runner/parallel.rs` |
| `axum` + `tower` middleware | `server/mod.rs` |
| `hyper` low-level HTTP | `server/github.rs` |
| `rustls` TLS | `server/tls.rs` |
| Lifetimes | `server/mod.rs`, `server/github.rs` |
| `tracing` + `#[instrument]` | `runner/shell.rs` |
| WebSocket | `server/dashboard.rs` |
| `rust-embed` | `server/dashboard.rs` |
| `sqlx` + SQLite | `db/mod.rs` |
| Phantom types | `db/mod.rs` (`RunId<Pending>`) |
| `sha2` + `memmap2` | `cache/mod.rs` |
| Procedural macros | `rustpipe-macros/src/lib.rs` |
| `reqwest` async HTTP | `notify/mod.rs` |
| `tonic` gRPC | `agent/agent.rs` |
| `unsafe` FFI | `agent/agent.rs` (`libc::setpgid`) |
| `statrs` statistics | `metrics/mod.rs` |
| `nom` parser combinators | `pipeline/dsl.rs` |
| Declarative macros | `runner/shell.rs` (`step_banner!`) |

---

## Running Tests

```bash
cargo test
```

24 integration tests covering: YAML/DSL parsing, validation, DAG scheduling, cycle detection, matrix expansion, secret masking, artifact caching, and file auto-detection.

---

## License

MIT
