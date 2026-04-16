# Milestone 8 — Artifact Caching ✅

## Goal
Skip stages whose inputs haven't changed since last run.

## Status: DONE

## Files Created
- `src/cache/mod.rs` — sha2 hashing, memmap2 file reading, cache hit/miss logic

## Files Modified
- `src/main.rs` — added `cache clear` subcommand, `--cache` flag on `run`
- `src/cli/mod.rs` — pre-flight cache check in run_pipeline(), audit log on cache hit
- `Cargo.toml` — added memmap2

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `sha2` content hashing | `cache/mod.rs` — Sha256 over stage name + step cmds + files |
| `memmap2` | `cache/mod.rs` — zero-copy file reading via `Mmap::map()` |
| `unsafe` | `cache/mod.rs` — `unsafe { Mmap::map(&file) }` (required by memmap2 API) |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Artifact caching | `.rustpipe-cache/<hash>` marker files |
| Cache invalidation | hash changes when step commands or source files change |
| Cache hit/miss display | colored terminal output per stage |
| Build optimization | unchanged stages skipped entirely |

## Commands
```bash
rustpipe run --cache                      # enable artifact cache
rustpipe run --cache --parallel           # cache + parallel DAG execution
rustpipe cache clear                      # wipe .rustpipe-cache/
```

## How It Works
1. Before execution, hash each stage: `sha256(stage_name + step_cmds + source_files)`
2. Check if `.rustpipe-cache/<hash>` exists → cache hit, skip stage
3. After stage passes → write `.rustpipe-cache/<hash>` marker
4. Next run with same inputs → instant skip
