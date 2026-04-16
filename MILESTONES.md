# ⚡ RustPipe — Project Milestones

GitOps CI/CD pipeline engine in Rust.

---

## Concept Coverage

### Rust Concepts Covered in This Project

| Concept | Milestone |
|---------|-----------|
| `clap` CLI | M1 |
| `serde` + `serde_yaml` | M1 |
| Enums + Pattern Matching | M1 |
| Traits + Trait Objects | M1 |
| Custom error types (`thiserror`) | M1 |
| `anyhow` error propagation | M1 |
| Ownership & Borrowing | M1–M2 |
| Closures + Iterators | M2 |
| `tokio` async runtime | M2 |
| `async/await` + Future | M2 |
| `tokio::process::Command` | M2 |
| Declarative macros | M2 |
| `Arc/Mutex/RwLock` | M3 |
| Channels (`mpsc`, `broadcast`) | M3–M4 |
| `tokio::select!` | M3 |
| `bollard` Docker SDK | M3 |
| `petgraph` DAG | M4 |
| `rayon` parallelism | M4 |
| Generics + Monomorphization | M4 |
| Typestate pattern | M4 |
| `axum` HTTP server | M5 |
| `tower` middleware | M5 |
| `hyper` low-level HTTP | M5 |
| `rustls` TLS | M5 |
| Lifetimes | M5 |
| `tracing` structured logging | M6 |
| WebSocket (`tungstenite`) | M6 |
| `sqlx` async SQL | M7 |
| Phantom types | M7 |
| `sha2` / content hashing | M8 |
| `memmap2` memory-mapped files | M8 |
| Procedural macros | M9 |
| `reqwest` async HTTP client | M10 |
| `tonic` gRPC | M12 |
| Unsafe FFI | M12 |
| `statrs` statistics | M13 |
| `nom` parsing | M15 |

### DevOps Concepts Covered in This Project

| Concept | Milestone |
|---------|-----------|
| CI/CD pipelines | M1 |
| Pipeline-as-code | M1 |
| GitOps workflow | M5 |
| Webhook-driven automation | M5 |
| Container isolation | M3 |
| Parallel job execution | M4 |
| Artifact caching | M8 |
| Secrets management | M9 |
| Audit logging | M9 |
| Retry with backoff | M10 |
| Distributed tracing | M6 |
| Dashboard visualization | M6 |
| Monitoring (metrics, alerts) | M13 |
| RBAC / Policy engine | M14 |
| State management | M7 |
| Drift detection | M15 |
| mTLS (runner ↔ server) | M12 |
| Load balancing (runner pool) | M12 |
| Service discovery (runners) | M12 |
| A/B testing (matrix builds) | M11 |

### Concepts NOT in RustPipe (covered in other projects)

| Concept | Project |
|---------|---------|
| Infrastructure as Code | 04-terraforge |
| Zero-trust networking | 05-vaultrust |
| PKI / Certificate Authority | 05-vaultrust |
| Canary / Blue-green deployments | 07-runedeploy |
| Feature flags | 07-runedeploy |
| Service mesh / Sidecar proxy | 08-rustmesh |
| Circuit breaker | 08-rustmesh |
| Time-series databases | 03-oxymon |
| Multi-cloud abstraction | 06-rustcloud |
| Chaos engineering | novel/01-chaosweaver |
| Container orchestration | 01-cargoforge |

---

## 🔴 MUST HAVE — Core Engine

### Milestone 1 — YAML Pipeline Parser + CLI Skeleton
**Goal:** Read a `.rustpipe.yml` file and print out the parsed pipeline structure.

**Rust:** `serde` + `serde_yaml`, `clap` derive macros, enums + pattern matching, traits, `thiserror`, `anyhow`
**DevOps:** Pipeline-as-code, CI/CD pipeline design

- [ ] Define `Pipeline`, `Stage`, `Step` structs with `serde` derives
- [ ] Model stage status as an enum (`Pending`, `Running`, `Passed`, `Failed`, `Skipped`)
- [ ] Define a `Validator` trait with `validate(&self) -> Result<()>` — implement for `Pipeline`, `Stage`, `Step`
- [ ] Parse a `.rustpipe.yml` file into those structs
- [ ] `rustpipe validate <file>` — validate pipeline YAML and report errors
- [ ] `rustpipe init` — scaffold a default `.rustpipe.yml` in current directory
- [ ] Pretty-print the parsed pipeline to terminal
- [ ] Handle missing fields, bad YAML, unknown keys with clear error messages using `thiserror`
- [ ] Use `anyhow` for error propagation in `main`

---

### Milestone 2 — Local Shell Execution
**Goal:** Actually run pipeline steps as shell commands on your machine.

**Rust:** `tokio` async runtime, `async/await`, `tokio::process::Command`, closures + iterators, declarative macros
**DevOps:** Sequential job execution, exit code handling, failure propagation

- [ ] Execute each step's `run:` command as a subprocess via `tokio::process::Command`
- [ ] Stream stdout/stderr live to terminal (not buffered)
- [ ] Capture exit codes — fail the stage if any step fails
- [ ] Sequential step execution within a stage
- [ ] `rustpipe run --pipeline <file>` CLI command
- [ ] Colored terminal output (green = pass, red = fail)
- [ ] Write a `pipeline_step!` declarative macro to reduce step-execution boilerplate
- [ ] Use iterator chaining to collect and filter steps before execution

---

### Milestone 3 — Docker-Isolated Stage Execution
**Goal:** Run each stage inside a Docker container matching `runs-on:` image.

**Rust:** `bollard` Docker SDK, `Arc/Mutex/RwLock`, `tokio::select!`, `mpsc` channels
**DevOps:** Container isolation, container lifecycle management, volume mounts, stage timeouts

- [ ] Pull Docker image if not present locally
- [ ] Create container per stage with workspace mounted as volume
- [ ] Execute steps inside the container
- [ ] Stream container logs to terminal via `mpsc` channel in real time
- [ ] Remove container after stage completes (pass or fail)
- [ ] Timeout per stage — use `tokio::select!` to race execution against a sleep timer, kill container if exceeded
- [ ] Share a `ContainerPool` behind `Arc<Mutex<...>>` for safe concurrent access

---

### Milestone 4 — DAG-Based Parallel Job Execution
**Goal:** Run independent stages in parallel, respecting `needs:` dependencies.

**Rust:** `petgraph`, topological sort, `tokio::spawn`, `rayon`, generics, typestate pattern
**DevOps:** Parallel job scheduling, dependency management, circular dependency detection

- [ ] Build a dependency graph from `needs:` fields using `petgraph`
- [ ] Topological sort to determine execution order
- [ ] Run stages with no dependencies in parallel using `tokio::spawn`
- [ ] Block dependent stages until their `needs:` stages finish using `broadcast` channel
- [ ] Detect and error on circular dependencies
- [ ] Show a visual dependency graph in terminal on `rustpipe validate`
- [ ] Use `rayon` for CPU-bound work (e.g., hashing all stage inputs before execution starts)
- [ ] Model pipeline execution state with the typestate pattern: `Pipeline<Validated>` → `Pipeline<Scheduled>` → `Pipeline<Running>`
- [ ] Make `Scheduler<T>` generic over the executor type

---

### Milestone 5 — Webhook Server + TLS
**Goal:** Trigger pipelines automatically on git push via webhooks, served over HTTPS.

**Rust:** `axum`, `tower` middleware, `hyper`, `rustls`, lifetimes
**DevOps:** GitOps workflow, webhook-driven automation, HMAC request authentication

- [ ] `rustpipe serve --port 9090` starts an HTTPS server using `rustls` with a self-signed cert
- [ ] `POST /webhook/github` endpoint receives GitHub push events
- [ ] Verify `X-Hub-Signature-256` HMAC header to authenticate requests
- [ ] Parse the event payload — extract branch, commit SHA, repo URL
- [ ] Trigger the correct pipeline based on branch rules
- [ ] `GET /api/v1/runs` — list recent pipeline runs and their status
- [ ] Add `tower` middleware layers: request logging, timeout, rate limiting
- [ ] Use `hyper` directly for the low-level HTTP client that calls back to GitHub API
- [ ] Use explicit lifetime annotations on request handler structs that borrow config

---

## 🟡 HIGH IMPACT — Makes It Impressive

### Milestone 6 — Web Dashboard + Live Log Streaming
**Goal:** Browser UI that shows live pipeline run status and streams logs in real time.

**Rust:** `tungstenite` WebSocket, `tracing` + `tracing-subscriber`, `rust-embed`
**DevOps:** Dashboard visualization, distributed tracing, structured logging

- [ ] Replace all `println!` with `tracing` macros (`info!`, `warn!`, `error!`)
- [ ] Add `tracing-subscriber` with JSON formatter for structured log output
- [ ] Embed a minimal HTML/JS dashboard using `rust-embed`
- [ ] `GET /dashboard` serves the UI
- [ ] `WS /ws/runs/:id/logs` streams live log lines to the browser via WebSocket
- [ ] Dashboard shows all active runs with stage status indicators
- [ ] Each pipeline run gets a `tracing` span — child spans per stage and step

---

### Milestone 7 — Pipeline Run History (Persistence)
**Goal:** Persist all run metadata and logs to SQLite.

**Rust:** `sqlx` + SQLite, async DB queries, phantom types
**DevOps:** State management, audit logging, run history

- [ ] Create SQLite DB with `runs`, `stages`, `steps` tables via `sqlx` migrations
- [ ] Save run metadata on start (pipeline name, trigger, commit SHA)
- [ ] Update status and duration on completion
- [ ] Store full logs per step in DB
- [ ] `rustpipe history` — list last N runs with status and duration
- [ ] `rustpipe logs <run-id>` — print full logs for a past run
- [ ] Use phantom types to distinguish `RunId<Pending>` vs `RunId<Completed>` at compile time
- [ ] Audit log: every state transition (start, pass, fail, cancel) written as an immutable row

---

### Milestone 8 — Artifact Caching
**Goal:** Skip stages whose inputs haven't changed since last run.

**Rust:** `sha2` content hashing, `memmap2` memory-mapped files
**DevOps:** Artifact caching, cache invalidation strategy, build optimization

- [ ] Define `artifact:` outputs in pipeline YAML
- [ ] Use `memmap2` to memory-map source files for fast hashing
- [ ] Hash the stage inputs (source files + step commands) with `sha2`
- [ ] Store artifacts in `.rustpipe-cache/` directory
- [ ] Restore cache on next run if hash matches — skip stage entirely
- [ ] `rustpipe cache clear` CLI command
- [ ] Show cache hit/miss in terminal output

---

### Milestone 9 — Secret Masking + RBAC
**Goal:** Secure secret injection and role-based access control on the API.

**Rust:** Procedural macros, log filtering, env var injection
**DevOps:** Secrets management, audit logging, RBAC / policy engine

- [ ] Define secrets in pipeline config or environment
- [ ] Inject secrets as env vars into container stages
- [ ] Intercept all log output and replace secret values with `***`
- [ ] Warn if a secret appears to be hardcoded in a step's `run:` command
- [ ] Never store raw secret values in the run history DB
- [ ] Write a `#[requires_role("admin")]` procedural macro attribute for API route handlers
- [ ] Implement a simple RBAC check: `viewer` can read runs, `operator` can trigger, `admin` can configure
- [ ] Every secret access and RBAC decision written to the audit log

---

### Milestone 10 — Notifications + Retry with Backoff
**Goal:** Notify on pipeline results and retry failed steps automatically.

**Rust:** `reqwest` async HTTP client
**DevOps:** Retry with backoff, Slack/Discord notifications, webhook integrations

- [ ] Configure notification channels in pipeline YAML
- [ ] Send Slack message on pipeline pass/fail with run summary
- [ ] Send Discord embed on pipeline pass/fail
- [ ] Generic webhook notification (POST JSON to any URL)
- [ ] Include commit SHA, branch, duration, and failed stage in message
- [ ] Retry failed steps with exponential backoff (`retry: attempts: 3, backoff: exponential`)
- [ ] Use `reqwest` with retry logic for all outbound HTTP calls (notifications + GitHub API)

---

## 🟢 ADD LATER — Advanced Features

### Milestone 11 — Matrix Builds
**Rust:** Advanced `tokio::spawn` fan-out, iterator combinators
**DevOps:** A/B testing via matrix, parallel job variants, fail-fast strategy

- [ ] Support `matrix:` key in stage definition
- [ ] Generate N parallel jobs from matrix combinations (e.g., 3 Rust versions × 2 OS = 6 jobs)
- [ ] Each matrix job runs in its own container with matrix vars as env vars
- [ ] Show matrix job results in a grid in the dashboard
- [ ] Fail the stage if any matrix job fails (configurable: `fail-fast`)

---

### Milestone 12 — Distributed Runners + mTLS
**Goal:** Remote runner agents that accept jobs over gRPC with mutual TLS.

**Rust:** `tonic` gRPC, `rustls` mTLS, unsafe FFI
**DevOps:** Distributed systems, load balancing, service discovery, mTLS, runner pool management

- [ ] Define `Runner` gRPC service in `.proto` file
- [ ] `rustpipe agent --server <url>` starts a remote runner agent
- [ ] Server assigns jobs to available agents via gRPC streaming
- [ ] Agent streams logs back to server in real time
- [ ] Runner labels — route jobs to specific runners (`runs-on: gpu`)
- [ ] Handle agent disconnection — reschedule job to another agent
- [ ] Secure all gRPC connections with mutual TLS (`rustls`)
- [ ] Server maintains a runner registry — basic service discovery (runners register on connect)
- [ ] Load balance jobs across available runners (round-robin)
- [ ] Use `unsafe` FFI to call a C library for a platform-specific runner capability (e.g., `libc` process isolation)

---

### Milestone 13 — Metrics + Observability
**Goal:** Expose Prometheus metrics and OpenTelemetry traces.

**Rust:** `statrs` statistics, OpenTelemetry SDK, `tracing-opentelemetry`
**DevOps:** Monitoring, metrics, alerting, distributed tracing, Prometheus

- [ ] Add OpenTelemetry exporter (Jaeger or OTLP)
- [ ] Create a trace span per pipeline run, child spans per stage and step
- [ ] Export metrics: runs/min, stage duration histogram, failure rate
- [ ] Expose `GET /metrics` in Prometheus text format
- [ ] Use `statrs` to compute p50/p95/p99 stage duration percentiles
- [ ] Alert rule examples: pipeline failure rate > 10% in 5 min window

---

### Milestone 14 — GitHub Status Checks + Full GitOps Loop
**Rust:** async API client, OAuth token handling
**DevOps:** Full GitOps loop, commit status checks, branch protection integration

- [ ] On pipeline start, post `pending` status to GitHub commit via API
- [ ] On stage completion, update status per stage
- [ ] On pipeline end, post final `success` or `failure` status
- [ ] Link status check back to the live dashboard run URL
- [ ] Support branch protection rules — block merge if pipeline fails

---

### Milestone 15 — Custom Pipeline DSL + Drift Detection
**Goal:** Support a custom `.rustpipe` DSL as an alternative to YAML, and detect pipeline config drift.

**Rust:** `nom` parser combinators
**DevOps:** Drift detection, pipeline-as-code evolution

- [ ] Write a `nom`-based parser for a minimal custom pipeline DSL
- [ ] DSL supports the same features as YAML format (stages, steps, needs, when)
- [ ] `rustpipe validate` works for both YAML and DSL formats
- [ ] Drift detection: compare current pipeline definition against the last-run version stored in DB
- [ ] Warn on `rustpipe run` if the pipeline definition has changed since the last successful run
- [ ] `--dry-run` flag — show what would execute without running anything
- [ ] Conditional steps — `when: branch == "main"`

---

### Milestone 16 — Zero-Warning Build + Library Crate ✅
**Goal:** Eliminate all compiler warnings and expose the codebase as a proper library crate.

**Rust:** Explicit lifetime annotations, `pub mod`, binary + library crate split, `#[allow(dead_code)]`
**DevOps:** Clean build gates, CI hygiene

- [x] Fix all 7 elided lifetime warnings in `dsl.rs` (`ParseResult<'_, T>`)
- [x] Remove all unused imports across the codebase
- [x] Add `#[allow(dead_code)]` to intentional API items not called from `main`
- [x] Create `src/lib.rs` exposing all modules as a public library crate
- [x] Add `[lib]` target to `Cargo.toml` alongside `[[bin]]`
- [x] Build produces zero warnings: `warning: rustpipe generated 0 warnings`

---

### Milestone 17 — Integration Test Suite ✅
**Goal:** Add a comprehensive integration test suite covering all core subsystems.

**Rust:** `#[test]`, external test crate (`tests/`), `assert_eq!`, `assert!`, `assert_ne!`, temp file I/O
**DevOps:** Test-driven bug discovery, CI test gates

- [x] 24 integration tests across 8 groups: YAML parsing, DSL parsing, validation, DAG scheduling, matrix expand, secrets, cache, file auto-detect
- [x] Discovered and fixed missing unknown-dependency check in `Pipeline::validate()`
- [x] Fixed doc comment code block in `dsl.rs` to prevent false doctest failures
- [x] All tests pass: `test result: ok. 24 passed; 0 failed`

---

## Priority Order

| Priority | Milestone | Key Concepts Unlocked |
|----------|-----------|----------------------|
| 1 | M1 — Parser + CLI | serde, clap, traits, enums, thiserror |
| 2 | M2 — Shell Execution | tokio, async/await, closures, declarative macros |
| 3 | M3 — Docker Isolation | bollard, Arc/Mutex, channels, tokio::select! |
| 4 | M4 — DAG Execution | petgraph, rayon, generics, typestate pattern |
| 5 | M5 — Webhook + TLS | axum, tower, hyper, rustls, lifetimes |
| 6 | M6 — Dashboard + Logs | WebSocket, tracing, structured logging |
| 7 | M7 — Run History | sqlx, phantom types, audit logging |
| 8 | M8 — Artifact Caching | memmap2, sha2, cache invalidation |
| 9 | M9 — Secret Masking + RBAC | procedural macros, RBAC, secrets management |
| 10 | M10 — Notifications + Retry | reqwest, retry with backoff |
| 11 | M11 — Matrix Builds | A/B testing, parallel variants |
| 12 | M12 — Distributed Runners | tonic gRPC, mTLS, service discovery, load balancing, unsafe FFI |
| 13 | M13 — Metrics | statrs, OpenTelemetry, Prometheus |
| 14 | M14 — GitHub Status Checks | Full GitOps loop |
| 15 | M15 — Custom DSL + Drift | nom parsing, drift detection |
| 16 | M16 — Zero-Warning Build + Library Crate | explicit lifetimes, `pub mod`, binary+library crate |
| 17 | M17 — Integration Test Suite | `#[test]`, external test crate, bug-driven fixes |
