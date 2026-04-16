# Milestone 2 — Local Shell Execution ✅

## Goal
Run pipeline steps as real shell commands, streaming output live.

## Status: DONE

## Files Created
- `src/runner/mod.rs` — runner module
- `src/runner/shell.rs` — execute(), run_command(), step_banner! macro

## Files Modified
- `src/cli/mod.rs` — added run_pipeline()
- `src/main.rs` — added `Run` subcommand, switched to `#[tokio::main]`
- `Cargo.toml` — added `tokio`

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `tokio` async runtime | `main.rs` — `#[tokio::main]` |
| `async/await` + Future | `shell.rs` — entire execution chain is async |
| `tokio::process::Command` | `shell.rs` — spawns real shell subprocesses |
| `AsyncBufReadExt` + `BufReader` | `shell.rs` — streams stdout/stderr line by line |
| Declarative macros | `shell.rs` — `step_banner!()` macro |
| Closures + Iterators | `shell.rs` — `.keys().collect()` |
| Labeled break (`break 'steps`) | `shell.rs` — early exit on step failure |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Sequential step execution | Steps run in order within a stage |
| Exit code propagation | Step fails → stage fails → pipeline stops |
| Live log streaming | Output appears line-by-line as it happens |

## Commands
```bash
rustpipe run                         # run .rustpipe.yml
rustpipe run --pipeline demo.yml     # run specific file
```
