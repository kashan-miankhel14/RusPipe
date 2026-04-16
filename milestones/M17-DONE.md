# Milestone 17 — Integration Test Suite ✅

## Goal
Add a comprehensive integration test suite covering all core subsystems. Tests must run with `cargo test` and produce zero failures.

## Status: DONE

## Files Created
- `tests/integration.rs` — 24 integration tests across 8 test groups

## Files Modified
- `src/pipeline/validator.rs` — added unknown `needs` dependency check to `Pipeline::validate()`
- `src/pipeline/dsl.rs` — fixed doc comment code block (```` ```text ```` instead of ```` ``` ````) to prevent false doctest failures

## Test Coverage

| Group | Tests | What's Covered |
|-------|-------|----------------|
| YAML parsing | 3 | pipeline name, stage count, `needs` field |
| DSL parsing | 3 | pipeline name, stage count, `needs` field |
| Validation | 3 | valid pipeline passes, empty name fails, unknown dependency fails |
| DAG scheduling | 3 | wave count, first-wave stage, cycle detection |
| Matrix expand | 3 | 2×2 = 4 combos, single axis, key presence |
| Secrets | 4 | mask replaces value, empty value skipped, hardcoded detected, clean command passes |
| Cache | 3 | hash determinism, hash differs on cmd change, write + check roundtrip |
| File auto-detect | 2 | `.yml` → YAML parser, `.rustpipe` → DSL parser |

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| Integration tests (`tests/`) | `tests/integration.rs` — external test crate accessing public API |
| `#[test]` | All 24 test functions |
| `assert_eq!` / `assert!` / `assert_ne!` | Throughout |
| `HashMap` construction in tests | Matrix expand tests |
| Temp file I/O in tests | File auto-detect tests — write to `/tmp`, clean up after |
| `unwrap()` vs `is_err()` | Positive path tests use `unwrap()`, negative path tests use `is_err()` |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Test-driven validation | Unknown dependency check was missing — discovered and fixed by writing the test first |
| Cache correctness | Tests verify SHA-256 hash determinism and filesystem roundtrip |
| Secret safety | Tests verify masking and hardcoded detection logic |
| DAG correctness | Cycle detection test ensures invalid pipelines are rejected |

## Bug Fixed
`Pipeline::validate()` did not check whether `needs:` referenced stages actually exist. Added cross-reference check — now returns `PipelineError::Validation` with message `"unknown dependency '<name>'"`.

## Commands
```bash
cargo test                          # run all 24 tests
cargo test test_dag                 # run only DAG tests
cargo test test_secret              # run only secret tests
```

## Result
```
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
