use crate::pipeline::dag::{TypedPipeline, Validated};
use crate::pipeline::model::StageStatus;
use crate::runner::{container, shell};
use colored::Colorize;
use rayon::prelude::*;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Hash all stage inputs in parallel using rayon before execution starts.
/// This is the CPU-bound work rayon is suited for.
fn hash_stage_inputs(pipeline: &crate::pipeline::model::Pipeline) -> Vec<(String, u64)> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pipeline
        .stages
        .par_iter()
        .map(|(name, stage)| {
            let mut h = DefaultHasher::new();
            name.hash(&mut h);
            stage.runs_on.hash(&mut h);
            for step in &stage.steps {
                step.run.hash(&mut h);
            }
            (name.clone(), h.finish())
        })
        .collect()
}

/// Execute pipeline with DAG-based parallel scheduling.
/// Stages in the same wave run concurrently via tokio::spawn.
pub async fn execute(pipeline: crate::pipeline::model::Pipeline, docker: bool, workspace: &str) -> anyhow::Result<()> {
    // Typestate: Validated → Scheduled → Running
    let typed = TypedPipeline::<Validated>::new(pipeline);
    let scheduled = typed.schedule().map_err(|e| anyhow::anyhow!(e))?;

    // Rayon: hash all stage inputs in parallel (CPU-bound pre-flight)
    let hashes = hash_stage_inputs(&scheduled.inner);
    println!("\n{} Stage input hashes:", "🔑".dimmed());
    for (name, hash) in &hashes {
        println!("   {} → {:x}", name.dimmed(), hash);
    }

    let running = scheduled.start();
    let pipeline = Arc::new(running.inner);

    println!("\n{} Running pipeline: {}", "⚡".yellow(), pipeline.name.cyan().bold());
    println!("   {} execution waves\n", running.order.len().to_string().yellow());

    // broadcast channel: completed stage names notify dependent stages
    let (tx, _) = broadcast::channel::<String>(64);

    for (wave_idx, wave) in running.order.iter().enumerate() {
        println!("{} Wave {}: [{}]", "→".blue(), wave_idx, wave.join(", ").cyan());

        let mut handles = vec![];

        for stage_name in wave {
            let stage_name = stage_name.clone();
            let pipeline = Arc::clone(&pipeline);
            let tx = tx.clone();
            let workspace = workspace.to_string();

            let handle = tokio::spawn(async move {
                let stage = &pipeline.stages[&stage_name];
                let status = if docker {
                    let docker_client = bollard::Docker::connect_with_local_defaults().unwrap();
                    let pool = container::new_pool();
                    container::execute_stage(&docker_client, pool, &stage_name, stage, &workspace)
                        .await
                        .unwrap_or(StageStatus::Failed)
                } else {
                    shell::execute_stage(&stage_name, stage).await.unwrap_or(StageStatus::Failed)
                };

                let _ = tx.send(stage_name.clone());
                (stage_name, status)
            });

            handles.push(handle);
        }

        // Wait for all stages in this wave
        for handle in handles {
            let (name, status) = handle.await?;
            if status == StageStatus::Failed {
                anyhow::bail!("Stage '{}' failed", name);
            }
        }
    }

    println!("{} Pipeline complete\n", "✅".green());
    Ok(())
}
