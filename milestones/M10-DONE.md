# Milestone 10 — Notifications + Retry with Backoff ✅

## Goal
Notify on pipeline results and retry failed steps automatically.

## Status: DONE

## Files Created
- `src/notify/mod.rs` — Slack, Discord, generic webhook notifications + exponential backoff retry

## Files Modified
- `src/cli/mod.rs` — notifications triggered after run completes
- `src/pipeline/model.rs` — added `notify` (NotifyConfig) and `retry` (RetryConfig) fields
- `Cargo.toml` — added reqwest (json feature)

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `reqwest` async HTTP client | `notify/mod.rs` — Client::new().post().json().send() |
| Exponential backoff | `with_retry()` — delay doubles each attempt |
| Closures + async | `with_retry()` — takes `FnMut() -> Fut` generic closure |
| Generics + Future bound | `with_retry<F, Fut>` — generic over async operation |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Retry with backoff | with_retry(3, ...) — 500ms → 1s → 2s |
| Slack notifications | notify_slack() — incoming webhook with run summary |
| Discord notifications | notify_discord() — embed with color-coded status |
| Generic webhook | notify_webhook() — POST JSON to any URL |
| Run summary | branch, commit, duration, failed stage included |

## Pipeline Config
```yaml
notify:
  slack:   https://hooks.slack.com/services/...
  discord: https://discord.com/api/webhooks/...
  webhook: https://my-server.com/ci-hook

stages:
  test:
    runs-on: rust:latest
    steps:
      - name: Run tests
        run: cargo test
        retry:
          attempts: 3
          backoff: exponential
```
