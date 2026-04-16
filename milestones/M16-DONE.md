# Milestone 16 — Zero-Warning Build + Library Crate ✅

## Goal
Eliminate all compiler warnings and expose the codebase as a proper library crate for testing and downstream use.

## Status: DONE

## Files Created
- `src/lib.rs` — public library entry point exposing all modules

## Files Modified
- `src/pipeline/dsl.rs` — fixed 7 elided lifetime warnings (`ParseResult<'_, T>`), removed unused imports (`map`, `space0`)
- `src/pipeline/dag.rs` — `#[allow(dead_code)]` on `Scheduler<T>`, `ShellExecutor`, `DockerExecutor` (intentional API)
- `src/pipeline/model.rs` — `#[allow(dead_code)]` on `StageStatus::Pending`
- `src/runner/shell.rs` — removed unused `warn` import, `#[allow(dead_code)]` on `execute()`
- `src/runner/matrix.rs` — `#[allow(dead_code)]` on `expand()`, `execute_matrix()`
- `src/runner/container.rs` — removed unused `HashMap` import
- `src/secrets/mod.rs` — `#[allow(dead_code)]` on `mask()`, `env_pairs()`
- `src/server/github.rs` — `#[allow(dead_code)]` on `post_commit_status()`, `parse_push_event()`, `GitOpsLoop`
- `src/server/rbac.rs` — removed unused `IntoResponse`, `RequestExt` imports
- `src/agent/registry.rs` — `#[allow(dead_code)]` on `RunnerRegistry`, `RunnerEntry`, `RegistryInner`
- `src/db/mod.rs` — `#[allow(dead_code)]` on `RunRow::finished_at`
- `src/metrics/mod.rs` — `#[allow(dead_code)]` on `percentiles()`, `print_percentiles()`
- `Cargo.toml` — added `[lib]` target pointing to `src/lib.rs`

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| Explicit lifetime annotations | `dsl.rs` — `ParseResult<'_, T>` on all nom parser functions |
| `#[allow(dead_code)]` | Intentional API items not called from `main` but part of the public surface |
| Library vs binary crate | `src/lib.rs` + `[[bin]]` in `Cargo.toml` — same code exposed as both |
| `pub mod` visibility | `lib.rs` — all modules re-exported for integration test access |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Clean build gates | Zero warnings = CI won't silently accumulate technical debt |
| Library crate separation | Enables `cargo test --test integration` without binary coupling |

## Result
```
warning: `rustpipe` (bin "rustpipe") generated 0 warnings
```
