# Milestone 15 — Custom Pipeline DSL + Drift Detection ✅

## Goal
Support a custom `.rustpipe` DSL as an alternative to YAML, and detect pipeline config drift.

## Status: DONE

## Files Created
- `src/pipeline/dsl.rs` — nom-based DSL parser
- `migrations/0002_pipeline_hashes.sql` — pipeline_hashes table

## Files Modified
- `src/pipeline/mod.rs` — auto-detect YAML vs DSL by file extension, file_hash()
- `src/db/mod.rs` — save_pipeline_hash(), get_pipeline_hash()
- `src/cli/mod.rs` — drift detection warning, --dry-run flag, hash saved after success
- `src/main.rs` — added --dry-run flag to run subcommand
- `Cargo.toml` — added nom

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `nom` parser combinators | `dsl.rs` — tag, take_while1, many0, opt, delimited, terminated |
| `IResult` | `dsl.rs` — nom's result type for all parsers |
| `many0` | `dsl.rs` — parse zero or more stages/steps |
| `opt` | `dsl.rs` — optional needs/when clauses |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Drift detection | SHA256 hash of pipeline file compared to last-run hash |
| Pipeline-as-code evolution | DSL and YAML both supported, auto-detected |
| --dry-run | Shows execution plan without running anything |
| Conditional steps | `when:` field parsed in both YAML and DSL |

## DSL Format
```
pipeline my-pipeline

stage lint
  runs-on rust:latest
  step "Run clippy"
    run cargo clippy -- -D warnings
  end
end

stage test
  runs-on rust:latest
  needs lint
  when branch == "main"
  step "Run tests"
    run cargo test --all
  end
end
```

## Commands
```bash
rustpipe validate pipeline.rustpipe     # validate DSL file
rustpipe run --pipeline pipeline.rustpipe  # run DSL pipeline
rustpipe run --dry-run                  # show plan without executing
# On changed pipeline: "⚠ Pipeline definition has changed since last successful run"
```
