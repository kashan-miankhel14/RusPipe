use crate::cache;
use crate::db;
use crate::metrics;
use crate::notify::{notify_discord, notify_slack, notify_webhook, RunSummary};
use crate::pipeline::{self, validator::Validator};
use crate::runner::{container, parallel, shell};
use crate::secrets;
use colored::Colorize;
use std::fs;
use std::time::Instant;

const DEFAULT_PIPELINE: &str = r#"name: my-pipeline
trigger:
  branches: [main, develop]
  events: [push, pull_request]

stages:
  lint:
    runs-on: rust:latest
    steps:
      - name: Run clippy
        run: cargo clippy -- -D warnings

  test:
    runs-on: rust:latest
    needs: [lint]
    steps:
      - name: Run tests
        run: cargo test --all

  build:
    runs-on: rust:latest
    needs: [test]
    steps:
      - name: Build release
        run: cargo build --release
        artifact: app-binary
"#;

const DB_PATH: &str = ".rustpipe.db";

pub fn run_validate(file: &str) {
    match pipeline::parse(file) {
        Err(e) => {
            eprintln!("{} {}", "✗ Parse error:".red().bold(), e);
            std::process::exit(1);
        }
        Ok(p) => match p.validate() {
            Err(e) => {
                eprintln!("{} {}", "✗ Validation error:".red().bold(), e);
                std::process::exit(1);
            }
            Ok(_) => {
                println!("{} Pipeline '{}' is valid\n", "✓".green().bold(), p.name.cyan());
                print_pipeline(&p);
                // M4: show visual dependency graph
                print_dag(&p);
            }
        },
    }
}

pub async fn run_pipeline(file: &str, docker: bool, par: bool, use_cache: bool, dry_run: bool) {
    let p = match pipeline::parse(file) {
        Err(e) => { eprintln!("{} {}", "✗".red().bold(), e); std::process::exit(1); }
        Ok(p) => p,
    };
    if let Err(e) = p.validate() {
        eprintln!("{} {}", "✗ Validation:".red().bold(), e);
        std::process::exit(1);
    }

    // M15: dry-run — show what would execute without running
    if dry_run {
        println!("{} Dry run — pipeline '{}'\n", "🔍".blue(), p.name.cyan().bold());
        for (name, stage) in &p.stages {
            println!("  {} {} [{}]", "→".blue(), name.cyan().bold(), stage.runs_on.yellow());
            if let Some(needs) = &stage.needs {
                println!("    needs: {}", needs.join(", ").dimmed());
            }
            for step in &stage.steps {
                println!("    {} {}: {}", "▶".dimmed(), step.name.white(), step.run.dimmed());
            }
        }
        return;
    }

    // M7: open DB and record run start
    let pool = db::open(DB_PATH).await.expect("failed to open DB");

    // M15: drift detection — compare current file hash to last stored hash
    if let Some(current_hash) = pipeline::file_hash(file) {
        if let Ok(Some(stored_hash)) = db::get_pipeline_hash(&pool, &p.name).await {
            if stored_hash != current_hash {
                println!(
                    "{} Pipeline definition has changed since last successful run (drift detected)",
                    "⚠".yellow().bold()
                );
            }
        }
    }

    let run_id = db::insert_run(&pool, &p.name, "local", "HEAD")
        .await
        .expect("failed to insert run");

    // M9: load secrets, warn on hardcoded values
    let secret_map = secrets::load_from_env();
    if !secret_map.is_empty() {
        println!("{} Loaded {} secret(s) from env", "🔐".blue(), secret_map.len());
        for (name, stage) in &p.stages {
            for step in &stage.steps {
                let hits = secrets::check_hardcoded(&step.run, &secret_map);
                for key in hits {
                    eprintln!("{} Secret '{}' appears hardcoded in step '{}' of stage '{}'",
                        "⚠".yellow().bold(), key, step.name, name);
                }
            }
        }
    }

    // M8: pre-flight cache check
    if use_cache {
        println!("\n{} Checking artifact cache...", "🔍".blue());
        for (name, stage) in &p.stages {
            let cmds: Vec<&str> = stage.steps.iter().map(|s| s.run.as_str()).collect();
            let hash = cache::stage_hash(name, &cmds, &[]);
            if cache::check(name, &hash) {
                let _ = db::audit(&pool, run_id.value, "cache_hit", Some(name)).await;
            }
        }
    }

    // M13: init metrics
    metrics::init();

    let workspace = std::env::current_dir().unwrap().to_string_lossy().to_string();
    let started = Instant::now();
    let result = if par {
        parallel::execute(p.clone(), docker, &workspace).await
    } else if docker {
        container::execute(&p, &workspace).await
    } else {
        shell::execute_with_db(&p, Some((&pool, run_id.value))).await
    };
    let duration_secs = started.elapsed().as_secs();

    // M13: record run metric
    let status_str = if result.is_ok() { "passed" } else { "failed" };
    metrics::record_run(status_str);
    metrics::record_stage_duration("total", duration_secs as f64);

    // M7: record run completion
    let _ = db::finish_run(&pool, run_id, status_str).await;

    // M15: save pipeline hash after successful run
    if result.is_ok() {
        if let Some(hash) = pipeline::file_hash(file) {
            let _ = db::save_pipeline_hash(&pool, &p.name, &hash).await;
        }
    }

    // M10: send notifications if configured
    if let Some(notify_cfg) = &p.notify {
        let summary = RunSummary {
            pipeline: &p.name,
            branch: "local",
            commit: "HEAD",
            status: status_str,
            duration_secs,
            failed_stage: result.as_ref().err().map(|_| "unknown"),
        };
        if let Some(url) = &notify_cfg.slack {
            let _ = notify_slack(url, &summary).await;
        }
        if let Some(url) = &notify_cfg.discord {
            let _ = notify_discord(url, &summary).await;
        }
        if let Some(url) = &notify_cfg.webhook {
            let _ = notify_webhook(url, &summary).await;
        }
    }

    if let Err(e) = result {
        eprintln!("{} {}", "✗ Execution failed:".red().bold(), e);
        std::process::exit(1);
    }
}

pub async fn run_history(limit: i64) {
    let pool = match db::open(DB_PATH).await {
        Ok(p) => p,
        Err(_) => { println!("{} No run history yet.", "!".yellow()); return; }
    };
    let runs = db::list_runs(&pool, limit).await.unwrap_or_default();
    if runs.is_empty() {
        println!("{} No runs recorded yet.", "!".yellow());
        return;
    }
    println!("{:<5} {:<20} {:<12} {:<10} {:<10} {}",
        "ID", "Pipeline", "Branch", "Status", "Commit", "Started");
    println!("{}", "─".repeat(75).dimmed());
    for r in runs {
        let status_colored = match r.status.as_str() {
            "passed" => r.status.green().to_string(),
            "failed" => r.status.red().to_string(),
            _        => r.status.yellow().to_string(),
        };
        println!("{:<5} {:<20} {:<12} {:<10} {:<10} {}",
            r.id,
            truncate(&r.pipeline, 19),
            truncate(&r.branch, 11),
            status_colored,
            &r.commit_sha[..8.min(r.commit_sha.len())],
            &r.started_at[..19],
        );
    }
}

pub async fn run_logs(run_id: i64) {
    let pool = match db::open(DB_PATH).await {
        Ok(p) => p,
        Err(_) => { eprintln!("{} No DB found.", "✗".red()); return; }
    };
    let logs = db::get_logs(&pool, run_id).await.unwrap_or_default();
    if logs.is_empty() {
        println!("{} No logs for run {}.", "!".yellow(), run_id);
        return;
    }
    println!("{} Logs for run {}\n", "📋".blue(), run_id.to_string().cyan().bold());
    for entry in logs {
        let code_color = if entry.exit_code == 0 { "✓".green() } else { "✗".red() };
        println!("{} [{} › {}] (exit: {})",
            code_color, entry.stage.cyan(), entry.step.white(), entry.exit_code);
        for line in entry.output.lines() {
            println!("    {}", line.dimmed());
        }
    }
}

pub fn run_init() {
    let path = ".rustpipe.yml";
    if std::path::Path::new(path).exists() {
        println!("{} {} already exists", "!".yellow().bold(), path);
        return;
    }
    fs::write(path, DEFAULT_PIPELINE).expect("failed to write .rustpipe.yml");
    println!("{} Created {}", "✓".green().bold(), path.cyan());
}

fn print_pipeline(p: &crate::pipeline::model::Pipeline) {
    println!("{}: {}", "Pipeline".bold(), p.name.cyan());
    if let Some(trigger) = &p.trigger {
        if let Some(branches) = &trigger.branches {
            println!("  {}: {}", "Branches".bold(), branches.join(", ").yellow());
        }
        if let Some(events) = &trigger.events {
            println!("  {}: {}", "Events".bold(), events.join(", ").yellow());
        }
    }
    println!("\n  {}:", "Stages".bold());
    for (name, stage) in &p.stages {
        println!("    {} [{}]", name.cyan().bold(), stage.runs_on.yellow());
        if let Some(needs) = &stage.needs {
            println!("      needs: {}", needs.join(", ").dimmed());
        }
        let step_names: Vec<&str> = stage.steps.iter().map(|s| s.name.as_str()).collect();
        println!("      steps: {}", step_names.join(" → ").dimmed());
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}…", &s[..max - 1]) }
}

/// M4: Print a visual ASCII dependency graph to terminal.
fn print_dag(p: &crate::pipeline::model::Pipeline) {
    println!("\n  {}:", "Dependency Graph".bold());
    for (name, stage) in &p.stages {
        match &stage.needs {
            None => println!("    {} {} (no deps)", "◉".green(), name.cyan().bold()),
            Some(deps) => {
                for dep in deps {
                    println!("    {} {} → {}", "◉".blue(), dep.yellow(), name.cyan().bold());
                }
            }
        }
    }
}
