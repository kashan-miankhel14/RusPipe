# Milestone 4 — DAG-Based Parallel Job Execution ✅

## Goal
Run independent stages in parallel, respecting `needs:` dependencies.

## Status: DONE

## Files Created
- `src/pipeline/dag.rs` — DAG builder, topological sort, wave grouping, typestate structs
- `src/runner/parallel.rs` — parallel executor using tokio::spawn per wave

## Files Modified
- `src/pipeline/mod.rs` — added dag module
- `src/runner/shell.rs` — extracted `execute_stage()` for reuse
- `src/runner/mod.rs` — added parallel module
- `src/cli/mod.rs` — run_pipeline() accepts `par: bool`
- `src/main.rs` — added `--parallel` flag
- `Cargo.toml` — added `petgraph`, `rayon`

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `petgraph` DiGraph | `dag.rs` — builds dependency graph from `needs:` fields |
| Topological sort | `dag.rs` — `petgraph::algo::toposort` detects order + cycles |
| `tokio::spawn` | `parallel.rs` — each stage in a wave spawns its own async task |
| `broadcast` channel | `parallel.rs` — completed stage names broadcast to dependents |
| `rayon` par_iter | `parallel.rs` — hashes all stage inputs in parallel (CPU-bound) |
| Generics | `dag.rs` — `TypedPipeline<State>` generic over state marker |
| Typestate pattern | `dag.rs` — `Validated` → `Scheduled` → `Running` enforced at compile time |
| `PhantomData` | `dag.rs` — carries state type without runtime cost |
| `Arc` | `parallel.rs` — pipeline shared across spawned tasks |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Parallel job scheduling | Stages in same wave run concurrently |
| Dependency management | `needs:` fields respected — wave ordering enforced |
| Circular dependency detection | `toposort` returns error on cycle |
| DAG execution model | Same model used by GitHub Actions, GitLab CI, Argo Workflows |

## Commands
```bash
rustpipe run --parallel                        # DAG parallel, shell mode
rustpipe run --parallel --docker               # DAG parallel, Docker mode
rustpipe run --parallel --pipeline demo.yml    # specific file
```

## How Waves Work
Given:
```
lint (no deps)   → wave 0
test (needs lint) → wave 1
build (needs test) → wave 2
deploy (needs build) → wave 3
```
Stages in the same wave run in parallel via tokio::spawn.
