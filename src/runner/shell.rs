use crate::pipeline::model::{Pipeline, Stage, StageStatus};
use colored::Colorize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{info, instrument};

macro_rules! step_banner {
    ($stage:expr, $step:expr) => {
        println!("\n  {} {} › {}", "▶".blue(), $stage.cyan().bold(), $step.white());
    };
}

async fn run_command(cmd: &str) -> anyhow::Result<i32> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await? {
            println!("    {}", line.dimmed());
        }
    }
    if let Some(stderr) = child.stderr.take() {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await? {
            eprintln!("    {}", line.yellow());
        }
    }

    let status = child.wait().await?;
    Ok(status.code().unwrap_or(1))
}

/// Execute a single stage — called by both sequential and parallel executors
#[instrument(skip(stage), fields(stage = %stage_name, image = %stage.runs_on))]
pub async fn execute_stage(stage_name: &str, stage: &Stage) -> anyhow::Result<StageStatus> {
    execute_stage_with_db(stage_name, stage, None).await
}

pub async fn execute_stage_with_db(
    stage_name: &str,
    stage: &Stage,
    db: Option<(&sqlx::SqlitePool, i64)>,
) -> anyhow::Result<StageStatus> {
    println!("{} Stage: {} [{}]", "┌".blue(), stage_name.cyan().bold(), stage.runs_on.yellow());

    // M15: conditional step evaluation — skip stage if `when:` condition is false
    if let Some(when) = &stage.when {
        let branch = std::env::var("RUSTPIPE_BRANCH").unwrap_or_default();
        let condition_met = eval_when(when, &branch);
        if !condition_met {
            println!("{} Stage {} {} (when: {})\n", "└".yellow(), stage_name.cyan(), "SKIPPED".yellow().bold(), when.dimmed());
            return Ok(StageStatus::Skipped);
        }
    }

    let mut status = StageStatus::Running;

    // M2: iterator chaining — collect only non-empty steps, then execute
    let steps: Vec<&crate::pipeline::model::Step> = stage.steps.iter()
        .filter(|s| !s.run.trim().is_empty())
        .collect();

    'steps: for step in steps {
        step_banner!(stage_name, step.name.as_str());

        // M10: retry with exponential backoff
        let attempts = step.retry.as_ref().map(|r| r.attempts).unwrap_or(1);
        let mut delay = std::time::Duration::from_millis(500);
        let mut code = 1;
        for attempt in 1..=attempts {
            code = run_command(&step.run).await?;
            if code == 0 { break; }
            if attempt < attempts {
                println!("  {} Retry {}/{} in {}ms", "↺".yellow(), attempt, attempts, delay.as_millis());
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }

        // M7: log step output to DB
        if let Some((pool, run_id)) = db {
            let _ = crate::db::log_step(pool, run_id, stage_name, &step.name, &step.run, code as i64).await;
        }

        if code == 0 {
            println!("  {} {}", "✓".green().bold(), step.name.green());
        } else {
            println!("  {} {} (exit code: {})", "✗".red().bold(), step.name.red(), code);
            status = StageStatus::Failed;
            break 'steps;
        }
    }

    match status {
        StageStatus::Failed => println!("{} Stage {} {}\n", "└".red(), stage_name.red().bold(), "FAILED".red().bold()),
        _ => {
            status = StageStatus::Passed;
            // M8: write cache marker after successful stage
            let cmds: Vec<&str> = stage.steps.iter().map(|s| s.run.as_str()).collect();
            let hash = crate::cache::stage_hash(stage_name, &cmds, &[]);
            let _ = crate::cache::write_cache(&hash);
            println!("{} Stage {} {}\n", "└".green(), stage_name.cyan().bold(), "PASSED".green().bold());
        }
    }

    Ok(status)
}

/// Evaluate a simple `when:` condition: "branch == \"main\""
fn eval_when(expr: &str, branch: &str) -> bool {
    // Support: branch == "value"  |  branch != "value"
    if let Some(rest) = expr.strip_prefix("branch == ") {
        let expected = rest.trim().trim_matches('"');
        return branch == expected;
    }
    if let Some(rest) = expr.strip_prefix("branch != ") {
        let expected = rest.trim().trim_matches('"');
        return branch != expected;
    }
    true // unknown condition → allow
}

/// Sequential execution (M2 path — no DAG)
#[allow(dead_code)]
#[instrument(skip(pipeline), fields(pipeline = %pipeline.name))]
pub async fn execute(pipeline: &Pipeline) -> anyhow::Result<()> {
    execute_with_db(pipeline, None).await
}

/// Sequential execution with optional DB logging (M7).
pub async fn execute_with_db(
    pipeline: &Pipeline,
    db: Option<(&sqlx::SqlitePool, i64)>,
) -> anyhow::Result<()> {
    info!("Pipeline started");
    println!("\n{} Running pipeline: {}\n", "⚡".yellow(), pipeline.name.cyan().bold());

    for (stage_name, stage) in &pipeline.stages {
        let status = execute_stage_with_db(stage_name, stage, db).await?;
        if status == StageStatus::Failed {
            anyhow::bail!("Stage '{}' failed", stage_name);
        }
    }

    info!("Pipeline complete");
    println!("{} Pipeline complete\n", "✅".green());
    Ok(())
}
