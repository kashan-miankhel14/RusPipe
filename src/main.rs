mod agent;
mod cache;
mod cli;
mod db;
mod error;
mod metrics;
mod notify;
mod pipeline;
mod runner;
mod secrets;
mod server;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustpipe", about = "GitOps CI/CD pipeline engine", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a pipeline YAML or DSL file
    Validate { file: String },
    /// Scaffold a default .rustpipe.yml in the current directory
    Init,
    /// Run a pipeline locally
    Run {
        #[arg(short, long, default_value = ".rustpipe.yml")]
        pipeline: String,
        #[arg(long)]
        docker: bool,
        #[arg(long)]
        parallel: bool,
        #[arg(long)]
        cache: bool,
        /// Show what would run without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Start the webhook + dashboard server (HTTPS)
    Serve {
        #[arg(short, long, default_value_t = 9090)]
        port: u16,
        #[arg(long, default_value = "changeme")]
        secret: String,
    },
    /// Start a remote runner agent
    Agent {
        #[arg(long, default_value = "https://localhost:9090")]
        server: String,
        #[arg(long, default_value = "agent-1")]
        id: String,
        #[arg(long, value_delimiter = ',')]
        labels: Vec<String>,
    },
    /// List last N pipeline runs from history
    History {
        #[arg(short, long, default_value_t = 10)]
        limit: i64,
    },
    /// Print full logs for a past run
    Logs { run_id: i64 },
    /// Manage artifact cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Clear all cached artifacts
    Clear,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => cli::run_validate(&file),
        Commands::Init => cli::run_init(),
        Commands::Run { pipeline, docker, parallel, cache, dry_run } => {
            cli::run_pipeline(&pipeline, docker, parallel, cache, dry_run).await
        }
        Commands::Serve { port, secret } => server::serve(port, secret).await?,
        Commands::Agent { server, id, labels } => agent::run_agent(&server, &id, labels).await?,
        Commands::History { limit } => cli::run_history(limit).await,
        Commands::Logs { run_id } => cli::run_logs(run_id).await,
        Commands::Cache { action: CacheAction::Clear } => cache::clear()?,
    }

    Ok(())
}
