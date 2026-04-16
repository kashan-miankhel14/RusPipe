# Milestone 7 — Pipeline Run History (Persistence) ✅

## Goal
Persist all run metadata and logs to SQLite.

## Status: DONE

## Files Created
- `src/db/mod.rs` — SQLite pool, CRUD helpers, phantom-typed RunId
- `migrations/0001_init.sql` — runs, audit_log, step_logs tables

## Files Modified
- `src/main.rs` — added `history`, `logs` subcommands
- `src/cli/mod.rs` — run_history(), run_logs(), DB integration in run_pipeline()
- `Cargo.toml` — added sqlx (sqlite, runtime-tokio, migrate, chrono), chrono

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `sqlx` + SQLite | `db/mod.rs` — async queries, connection pool |
| `sqlx::migrate!` | `db/mod.rs` — runs migrations from `./migrations/` |
| Phantom types | `db/mod.rs` — `RunId<Pending>` vs `RunId<Completed>` |
| `PhantomData` | `db/mod.rs` — zero-cost state marker |
| Typestate transition | `finish_run()` consumes `RunId<Pending>`, returns `RunId<Completed>` |
| `chrono` | `db/mod.rs` — RFC3339 timestamps |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| State management | run status tracked: pending → passed/failed |
| Audit logging | every state transition written as immutable audit_log row |
| Run history | `rustpipe history` lists last N runs |
| Log retrieval | `rustpipe logs <id>` prints stored step output |

## Commands
```bash
rustpipe run --pipeline .rustpipe.yml     # records run to DB
rustpipe history                          # list last 10 runs
rustpipe history --limit 25              # list last 25 runs
rustpipe logs 1                           # print logs for run #1
```
