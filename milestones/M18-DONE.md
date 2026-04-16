# Milestone 18 — GitHub Actions CI + Webhook → Pipeline Execution ✅

## Goal
Make the project test itself on every push via GitHub Actions, and wire the webhook server to actually execute a pipeline when a GitHub push event arrives.

## Status: DONE

## Files Created
- `.github/workflows/ci.yml` — GitHub Actions workflow: clippy, tests, release build

## Files Modified
- `src/server/mod.rs` — webhook handler now spawns a `tokio::spawn` task that parses `.rustpipe.yml`, inserts a DB run, executes the pipeline via `shell::execute_with_db`, and marks it passed/failed

## GitHub Actions Workflow
```yaml
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - cargo clippy -- -D warnings   # zero warnings enforced
      - cargo test                    # 24 integration tests
      - cargo build --release         # release binary
```

Every push to `main` or `develop` runs the full CI pipeline. PRs are blocked if clippy or tests fail.

## Webhook → Pipeline Execution Flow
```
GitHub push event
  → POST /webhook/github
  → HMAC-SHA256 signature verified
  → branch + commit extracted
  → tokio::spawn (non-blocking)
      → parse .rustpipe.yml
      → db::insert_run()
      → shell::execute_with_db()
      → db::finish_run("passed" | "failed")
```

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `tokio::spawn` | `server/mod.rs` — fire-and-forget background task |
| `drop(runs)` | Release `MutexGuard` before spawning to avoid holding lock across await |
| GitHub Actions YAML | `.github/workflows/ci.yml` |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| CI pipeline (self-hosted) | GitHub Actions runs `cargo test` on every push |
| Webhook-triggered execution | Push event → pipeline run, fully automated |
| Non-blocking job dispatch | `tokio::spawn` returns immediately, pipeline runs async |
| Audit trail | Every webhook-triggered run is recorded in SQLite with branch + commit SHA |

## Commands
```bash
# Trigger manually via curl (test webhook locally)
curl -k -X POST https://localhost:9090/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<hmac>" \
  -d '{"ref":"refs/heads/main","after":"abc123","repository":{"full_name":"user/repo"}}'

# Check run was recorded
./target/release/rustpipe history
```
