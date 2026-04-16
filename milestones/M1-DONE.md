# Milestone 1 — YAML Pipeline Parser + CLI Skeleton ✅

## Goal
Read a `.rustpipe.yml` file and print out the parsed pipeline structure.

## Status: DONE

## Files Created
- `src/error.rs` — custom error types
- `src/pipeline/model.rs` — Pipeline, Stage, Step structs + StageStatus enum
- `src/pipeline/validator.rs` — Validator trait
- `src/pipeline/mod.rs` — parse() function
- `src/cli/mod.rs` — run_validate(), run_init()
- `src/main.rs` — clap CLI wiring

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `serde` + `serde_yaml` | `model.rs` — structs deserialize from YAML |
| `clap` derive macros | `main.rs` — `Cli`, `Commands` enums |
| Enums + Pattern Matching | `model.rs` — `StageStatus`, `main.rs` — `Commands` match |
| Traits + Trait Objects | `validator.rs` — `Validator` trait on `Pipeline`, `Stage`, `Step` |
| `thiserror` | `error.rs` — `PipelineError` with 3 typed variants |
| `anyhow` | `main.rs` — `main() -> Result<()>` |
| Ownership & Borrowing | Throughout parse/validate chain |
| Closures + Iterators | `cli/mod.rs` — `.iter().map().collect()` for step names |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Pipeline-as-code | `.rustpipe.yml` defines entire CI/CD declaratively |
| CI/CD pipeline structure | Stages with `needs:`, `runs-on:`, `artifact:` |

## Commands
```bash
rustpipe init                        # scaffold .rustpipe.yml
rustpipe validate .rustpipe.yml      # validate and pretty-print
```
