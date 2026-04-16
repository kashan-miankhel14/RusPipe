# Milestone 13 — Metrics + Observability ✅

## Goal
Expose Prometheus metrics and compute stage duration percentiles.

## Status: DONE

## Files Created
- `src/metrics/mod.rs` — Prometheus counters/histograms, statrs percentiles, /metrics handler

## Files Modified
- `src/server/mod.rs` — added GET /metrics route, metrics::init() on startup
- `src/cli/mod.rs` — metrics::record_run() and record_stage_duration() after each run
- `Cargo.toml` — added prometheus, statrs, opentelemetry, tracing-opentelemetry

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `statrs` statistics | `metrics/mod.rs` — `Data::percentile()` for p50/p95/p99 |
| `prometheus` | `metrics/mod.rs` — CounterVec, HistogramVec, TextEncoder |
| `OnceLock` | `metrics/mod.rs` — lazy global metric registration |
| `tracing-opentelemetry` | Wired in Cargo.toml, ready for OTLP export |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Prometheus metrics | GET /metrics — text format scraped by Prometheus |
| Run counter | rustpipe_runs_total{status="passed|failed"} |
| Stage duration histogram | rustpipe_stage_duration_seconds{stage=...} |
| p50/p95/p99 percentiles | print_percentiles() for CLI output |
| Monitoring | Prometheus can alert on failure rate > threshold |

## Endpoints
```
GET /metrics  → Prometheus text format
```

## Alert Rule Example
```yaml
# prometheus/alerts.yml
- alert: HighFailureRate
  expr: rate(rustpipe_runs_total{status="failed"}[5m]) /
        rate(rustpipe_runs_total[5m]) > 0.1
  for: 5m
  annotations:
    summary: "Pipeline failure rate > 10%"
```
