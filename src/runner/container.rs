use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use colored::Colorize;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::time::{timeout, Duration};

use crate::pipeline::model::{Pipeline, Stage, StageStatus};

/// Shared pool tracking active container IDs — Arc<Mutex<>> for safe concurrent access
pub type ContainerPool = Arc<Mutex<Vec<String>>>;

pub fn new_pool() -> ContainerPool {
    Arc::new(Mutex::new(Vec::new()))
}

/// Pull image if not present, then run all steps inside a container.
pub async fn execute_stage(
    docker: &Docker,
    pool: ContainerPool,
    stage_name: &str,
    stage: &Stage,
    workspace: &str,
) -> anyhow::Result<StageStatus> {
    println!(
        "\n{} Stage: {} [{}]",
        "┌".blue(),
        stage_name.cyan().bold(),
        stage.runs_on.yellow()
    );

    // Pull image
    pull_image(docker, &stage.runs_on).await?;

    // Build the shell command: chain all steps with && so any failure stops execution
    let combined_cmd = stage
        .steps
        .iter()
        .map(|s| {
            println!("  {} {}", "▶".blue(), s.name.white());
            s.run.as_str()
        })
        .collect::<Vec<_>>()
        .join(" && ");

    let container_name = format!("rustpipe-{}-{}", stage_name, uuid_short());

    // Create container with workspace mounted
    let config = Config {
        image: Some(stage.runs_on.as_str()),
        cmd: Some(vec!["sh", "-c", &combined_cmd]),
        working_dir: Some("/workspace"),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(vec![format!("{}:/workspace", workspace)]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container = docker
        .create_container(
            Some(CreateContainerOptions { name: &container_name, platform: None }),
            config,
        )
        .await?;

    // Register in pool so it can be cleaned up on panic/cancel
    pool.lock().unwrap().push(container.id.clone());

    docker.start_container(&container.id, None::<StartContainerOptions<String>>).await?;

    // Stream logs with timeout via tokio::select!
    let log_result = timeout(
        Duration::from_secs(stage.timeout_secs.unwrap_or(300)),
        stream_logs(docker, &container.id),
    )
    .await;

    let status = match log_result {
        Ok(Ok(_)) => {
            // Wait for exit code
            let mut wait_stream = docker.wait_container(
                &container.id,
                Some(WaitContainerOptions { condition: "not-running" }),
            );
            let exit_code = wait_stream
                .next()
                .await
                .and_then(|r| r.ok())
                .map(|r| r.status_code)
                .unwrap_or(1);

            if exit_code == 0 { StageStatus::Passed } else { StageStatus::Failed }
        }
        Ok(Err(e)) => { eprintln!("  {} Log error: {}", "!".yellow(), e); StageStatus::Failed }
        Err(_) => {
            eprintln!("  {} Stage timed out", "✗".red().bold());
            StageStatus::Failed
        }
    };

    // Always remove container
    let _ = docker
        .remove_container(
            &container.id,
            Some(RemoveContainerOptions { force: true, ..Default::default() }),
        )
        .await;

    pool.lock().unwrap().retain(|id| id != &container.id);

    match &status {
        StageStatus::Passed => println!("{} Stage {} {}\n", "└".green(), stage_name.cyan().bold(), "PASSED".green().bold()),
        _ => println!("{} Stage {} {}\n", "└".red(), stage_name.red().bold(), "FAILED".red().bold()),
    }

    Ok(status)
}

async fn pull_image(docker: &Docker, image: &str) -> anyhow::Result<()> {
    print!("  {} Pulling {}... ", "↓".blue(), image.yellow());
    let mut stream = docker.create_image(
        Some(CreateImageOptions { from_image: image, ..Default::default() }),
        None,
        None,
    );
    while stream.next().await.is_some() {} // drain stream
    println!("{}", "done".green());
    Ok(())
}

async fn stream_logs(docker: &Docker, container_id: &str) -> anyhow::Result<()> {
    let mut logs = docker.logs(
        container_id,
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );
    while let Some(msg) = logs.next().await {
        match msg {
            Ok(output) => print!("    {}", output.to_string().dimmed()),
            Err(e) => eprintln!("    {}", e.to_string().yellow()),
        }
    }
    Ok(())
}

/// Execute all stages using Docker isolation
pub async fn execute(pipeline: &Pipeline, workspace: &str) -> anyhow::Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    let pool = new_pool();

    println!("\n{} Running pipeline: {}\n", "⚡".yellow(), pipeline.name.cyan().bold());

    for (stage_name, stage) in &pipeline.stages {
        let status = execute_stage(&docker, Arc::clone(&pool), stage_name, stage, workspace).await?;
        if status == StageStatus::Failed {
            anyhow::bail!("Stage '{}' failed", stage_name);
        }
    }

    println!("{} Pipeline complete\n", "✅".green());
    Ok(())
}

fn uuid_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    format!("{:x}", t)
}
