# Milestone 3 — Docker-Isolated Stage Execution ✅

## Goal
Run each pipeline stage inside its own Docker container.

## Status: DONE

## Files Created
- `src/runner/container.rs` — Docker executor, ContainerPool, pull/stream/cleanup logic

## Files Modified
- `src/runner/mod.rs` — added container module
- `src/pipeline/model.rs` — added `timeout_secs` to Stage
- `src/cli/mod.rs` — run_pipeline() now accepts `docker: bool`
- `src/main.rs` — added `--docker` flag to `run` subcommand
- `Cargo.toml` — added `bollard`, `futures-util`

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `bollard` Docker SDK | `container.rs` — create/start/log/remove containers |
| `Arc<Mutex<Vec<String>>>` | `container.rs` — `ContainerPool` shared across async tasks |
| `tokio::time::timeout` | `container.rs` — kills stage if it exceeds `timeout_secs` |
| `tokio::select!` (via timeout) | `container.rs` — races execution against timer |
| `mpsc` / stream channels | `container.rs` — `futures_util::StreamExt` drains log stream |
| `futures_util::StreamExt` | `container.rs` — `.next().await` on bollard streams |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Container isolation | Each stage runs in its own fresh Docker container |
| Volume mounts | Workspace mounted as `/workspace` inside container |
| Container lifecycle | Pull → Create → Start → Stream logs → Wait → Remove |
| Stage timeouts | Container killed if stage exceeds `timeout_secs` (default 300s) |
| Image management | Auto-pulls image if not present locally |

## Commands
```bash
rustpipe run --docker                          # run with Docker isolation
rustpipe run --pipeline demo.yml --docker      # specific file + Docker
```

## Example Pipeline with Timeout
```yaml
stages:
  test:
    runs-on: rust:latest
    timeout_secs: 120
    steps:
      - name: Run tests
        run: cargo test
```
