/// M11: Matrix builds — fan-out a stage across all combinations of matrix variables.
/// Each combination runs as an independent tokio task (parallel).
use crate::pipeline::model::{Stage, StageStatus};
use crate::runner::shell;
use colored::Colorize;
use std::collections::HashMap;

/// Generate all combinations of matrix variable values.
/// e.g. {rust: [stable, beta], os: [linux, macos]} → 4 combinations
#[allow(dead_code)]
pub fn expand(matrix: &HashMap<String, Vec<String>>) -> Vec<HashMap<String, String>> {
    let mut result: Vec<HashMap<String, String>> = vec![HashMap::new()];
    for (key, values) in matrix {
        result = values
            .iter()
            .flat_map(|v| {
                result.iter().map(move |existing| {
                    let mut combo = existing.clone();
                    combo.insert(key.clone(), v.clone());
                    combo
                })
            })
            .collect();
    }
    result
}

/// Run a stage across all matrix combinations in parallel via tokio::spawn.
/// Returns Failed if any job fails (respects fail_fast flag).
#[allow(dead_code)]
pub async fn execute_matrix(
    stage_name: &str,
    stage: &Stage,
    matrix: &HashMap<String, Vec<String>>,
) -> anyhow::Result<StageStatus> {
    let combos = expand(matrix);
    let total = combos.len();
    println!(
        "\n{} Matrix stage {} — {} combinations",
        "⊞".blue(),
        stage_name.cyan().bold(),
        total.to_string().yellow()
    );

    // Fan-out: spawn one task per combination
    let handles: Vec<_> = combos
        .into_iter()
        .enumerate()
        .map(|(i, vars)| {
            let label = vars
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            let stage_name = stage_name.to_string();
            let stage = stage.clone();
            tokio::spawn(async move {
                println!("  {} Job {}/{}: [{}]", "→".blue(), i + 1, total, label.cyan());
                // Inject matrix vars as env — in shell mode we prefix the command
                let env_prefix = vars
                    .iter()
                    .map(|(k, v)| format!("{}={}", k.to_uppercase(), v))
                    .collect::<Vec<_>>()
                    .join(" ");
                // Build a modified stage with env-prefixed step commands
                let mut patched = stage.clone();
                for step in &mut patched.steps {
                    step.run = format!("{} {}", env_prefix, step.run);
                }
                let status = shell::execute_stage(&stage_name, &patched)
                    .await
                    .unwrap_or(StageStatus::Failed);
                (label, status)
            })
        })
        .collect();

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await?);
    }

    // Print grid summary
    println!("\n  {} Matrix results:", "⊞".blue());
    let mut any_failed = false;
    for (label, status) in &results {
        let icon = if *status == StageStatus::Passed { "✓".green() } else { "✗".red() };
        println!("    {} [{}]", icon, label.dimmed());
        if *status == StageStatus::Failed {
            any_failed = true;
            if stage.fail_fast {
                println!("  {} fail-fast: aborting remaining jobs", "!".yellow().bold());
                break;
            }
        }
    }

    if any_failed {
        Ok(StageStatus::Failed)
    } else {
        Ok(StageStatus::Passed)
    }
}
