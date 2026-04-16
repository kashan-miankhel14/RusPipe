use std::marker::PhantomData;

use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

// --- Phantom type markers ---
pub struct Pending;
pub struct Completed;

/// Typed run ID — distinguishes pending vs completed runs at compile time.
pub struct RunId<State> {
    pub value: i64,
    _state: PhantomData<State>,
}

impl RunId<Pending> {
    pub fn new(id: i64) -> Self {
        Self { value: id, _state: PhantomData }
    }
    /// Transition: Pending → Completed
    pub fn complete(self) -> RunId<Completed> {
        RunId { value: self.value, _state: PhantomData }
    }
}

/// Open (or create) the SQLite DB and run migrations.
pub async fn open(path: &str) -> Result<SqlitePool> {
    let url = format!("sqlite://{}?mode=rwc", path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

/// Insert a new run row, return a `RunId<Pending>`.
pub async fn insert_run(
    pool: &SqlitePool,
    pipeline: &str,
    branch: &str,
    commit_sha: &str,
) -> Result<RunId<Pending>> {
    let now = Utc::now().to_rfc3339();
    let id = sqlx::query(
        "INSERT INTO runs (pipeline, branch, commit_sha, status, started_at) VALUES (?, ?, ?, 'pending', ?)",
    )
    .bind(pipeline)
    .bind(branch)
    .bind(commit_sha)
    .bind(&now)
    .execute(pool)
    .await?
    .last_insert_rowid();

    audit(pool, id, "run_started", Some(&format!("pipeline={}", pipeline))).await?;
    Ok(RunId::new(id))
}

/// Mark a run as finished, transition RunId Pending → Completed.
pub async fn finish_run(
    pool: &SqlitePool,
    run_id: RunId<Pending>,
    status: &str,
) -> Result<RunId<Completed>> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE runs SET status = ?, finished_at = ? WHERE id = ?")
        .bind(status)
        .bind(&now)
        .bind(run_id.value)
        .execute(pool)
        .await?;

    audit(pool, run_id.value, "run_finished", Some(status)).await?;
    Ok(run_id.complete())
}

/// Append a step log entry.
pub async fn log_step(
    pool: &SqlitePool,
    run_id: i64,
    stage: &str,
    step: &str,
    output: &str,
    exit_code: i64,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO step_logs (run_id, stage, step, output, exit_code, logged_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(stage)
    .bind(step)
    .bind(output)
    .bind(exit_code)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Write an immutable audit event row.
pub async fn audit(pool: &SqlitePool, run_id: i64, event: &str, detail: Option<&str>) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO audit_log (run_id, event, detail, occurred_at) VALUES (?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(event)
    .bind(detail)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

// --- Query helpers for CLI ---

pub struct RunRow {
    pub id: i64,
    pub pipeline: String,
    pub branch: String,
    pub commit_sha: String,
    pub status: String,
    pub started_at: String,
    #[allow(dead_code)]
    pub finished_at: Option<String>,
}

pub async fn list_runs(pool: &SqlitePool, limit: i64) -> Result<Vec<RunRow>> {
    let rows = sqlx::query(
        "SELECT id, pipeline, branch, commit_sha, status, started_at, finished_at
         FROM runs ORDER BY id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| RunRow {
            id: r.get("id"),
            pipeline: r.get("pipeline"),
            branch: r.get("branch"),
            commit_sha: r.get("commit_sha"),
            status: r.get("status"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
        })
        .collect())
}

pub struct StepLogRow {
    pub stage: String,
    pub step: String,
    pub output: String,
    pub exit_code: i64,
}

pub async fn get_logs(pool: &SqlitePool, run_id: i64) -> Result<Vec<StepLogRow>> {
    let rows = sqlx::query(
        "SELECT stage, step, output, exit_code FROM step_logs WHERE run_id = ? ORDER BY id",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| StepLogRow {
            stage: r.get("stage"),
            step: r.get("step"),
            output: r.get("output"),
            exit_code: r.get("exit_code"),
        })
        .collect())
}

/// M15: Store pipeline definition hash for drift detection.
pub async fn save_pipeline_hash(pool: &SqlitePool, pipeline: &str, hash: &str) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO pipeline_hashes (pipeline, hash, saved_at) VALUES (?, ?, ?)",
    )
    .bind(pipeline)
    .bind(hash)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_pipeline_hash(pool: &SqlitePool, pipeline: &str) -> Result<Option<String>> {
    let row = sqlx::query("SELECT hash FROM pipeline_hashes WHERE pipeline = ?")
        .bind(pipeline)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get("hash")))
}
