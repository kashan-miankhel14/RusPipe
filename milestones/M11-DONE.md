# Milestone 11 — Matrix Builds ✅

## Goal
Fan-out a stage across all combinations of matrix variables, running each in parallel.

## Status: DONE

## Files Created
- `src/runner/matrix.rs` — matrix expansion, tokio::spawn fan-out, fail-fast support

## Files Modified
- `src/pipeline/model.rs` — added `matrix` and `fail_fast` fields to Stage
- `src/runner/mod.rs` — added matrix module

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `tokio::spawn` fan-out | `matrix.rs` — one task per matrix combination |
| Iterator combinators | `expand()` — flat_map + clone to generate all combos |
| `Vec` collect + join | `matrix.rs` — label string from combo map |
| Closures | `expand()` — closure captures key/value per iteration |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| A/B testing via matrix | e.g. rust: [stable, beta] × os: [linux, macos] = 4 jobs |
| Parallel job variants | each combo runs as independent tokio task |
| fail-fast strategy | stage aborts remaining jobs on first failure if fail_fast: true |

## Pipeline Config
```yaml
stages:
  test:
    runs-on: rust:latest
    fail-fast: true
    matrix:
      rust: [stable, beta, nightly]
      os: [linux, macos]
    steps:
      - name: Run tests
        run: cargo test
```
Generates 6 parallel jobs: stable/linux, stable/macos, beta/linux, beta/macos, nightly/linux, nightly/macos
