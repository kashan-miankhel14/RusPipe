/// M13: Prometheus metrics + statrs percentile computation.
/// Exposes GET /metrics in Prometheus text format.
use axum::{http::StatusCode, response::IntoResponse};
use colored::Colorize;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, TextEncoder,
};
use statrs::statistics::OrderStatistics;
use std::sync::OnceLock;
use tracing::info;

static RUN_COUNTER: OnceLock<CounterVec> = OnceLock::new();
static STAGE_DURATION: OnceLock<HistogramVec> = OnceLock::new();

pub fn init() {
    RUN_COUNTER.get_or_init(|| {
        register_counter_vec!(
            "rustpipe_runs_total",
            "Total pipeline runs by status",
            &["status"]
        )
        .unwrap()
    });
    STAGE_DURATION.get_or_init(|| {
        register_histogram_vec!(
            "rustpipe_stage_duration_seconds",
            "Stage execution duration in seconds",
            &["stage"],
            vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
        )
        .unwrap()
    });
    info!("Prometheus metrics initialized");
}

pub fn record_run(status: &str) {
    if let Some(c) = RUN_COUNTER.get() {
        c.with_label_values(&[status]).inc();
    }
}

pub fn record_stage_duration(stage: &str, secs: f64) {
    if let Some(h) = STAGE_DURATION.get() {
        h.with_label_values(&[stage]).observe(secs);
    }
}

/// Compute p50/p95/p99 from a slice of durations using statrs OrderStatistics.
#[allow(dead_code)]
pub fn percentiles(durations: &[f64]) -> (f64, f64, f64) {
    if durations.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut data = statrs::statistics::Data::new(durations.to_vec());
    let p50 = data.percentile(50);
    let p95 = data.percentile(95);
    let p99 = data.percentile(99);
    (p50, p95, p99)
}

/// Axum handler: GET /metrics — Prometheus text format.
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buf = Vec::new();
    match encoder.encode(&metric_families, &mut buf) {
        Ok(_) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            buf,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Print a percentile summary to terminal.
#[allow(dead_code)]
pub fn print_percentiles(stage: &str, durations: &[f64]) {
    let (p50, p95, p99) = percentiles(durations);
    println!(
        "  {} {} — p50: {:.2}s  p95: {:.2}s  p99: {:.2}s",
        "📊".blue(),
        stage.cyan(),
        p50, p95, p99
    );
}
