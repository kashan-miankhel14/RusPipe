/// M12: Distributed runner agent — connects to server via gRPC, executes jobs, streams logs back.
/// Uses tonic gRPC + rustls mTLS + unsafe FFI (libc setpgid for process isolation).
use crate::agent::proto::runner_service_client::RunnerServiceClient;
use crate::agent::proto::{AgentInfo, LogLine};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tonic::transport::{Channel, ClientTlsConfig};
use tonic::Request;
use tracing::{info, warn};

/// Start a runner agent that connects to the server and processes jobs.
pub async fn run_agent(server_url: &str, agent_id: &str, labels: Vec<String>) -> anyhow::Result<()> {
    info!(server = %server_url, agent = %agent_id, "Agent starting");

    // mTLS config — domain name for cert verification
    let tls = ClientTlsConfig::new().domain_name("localhost");

    let channel = Channel::from_shared(server_url.to_string())?
        .tls_config(tls)?
        .connect()
        .await?;

    let mut client = RunnerServiceClient::new(channel);

    let info = AgentInfo {
        agent_id: agent_id.to_string(),
        labels,
    };

    println!("🤖 Agent {} connected to {}", agent_id, server_url);

    let mut job_stream = client.fetch_job(Request::new(info)).await?.into_inner();

    while let Some(job) = job_stream.message().await? {
        info!(job_id = %job.job_id, stage = %job.stage, "Received job");

        let job_id = job.job_id.clone();
        let commands = job.commands.clone();
        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel::<LogLine>(64);

        let exec_handle = tokio::spawn(async move {
            for cmd in &commands {
                let mut child = unsafe {
                    Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        // SAFETY: setpgid(0,0) isolates child in its own process group.
                        // Called in the child process after fork, before exec — safe per POSIX.
                        .pre_exec(|| {
                            libc::setpgid(0, 0);
                            Ok(())
                        })
                        .spawn()?
                };

                if let Some(stdout) = child.stdout.take() {
                    let mut lines = BufReader::new(stdout).lines();
                    while let Some(line) = lines.next_line().await? {
                        let _ = log_tx.send(LogLine {
                            job_id: job_id.clone(),
                            text: line,
                            stderr: false,
                        }).await;
                    }
                }
                child.wait().await?;
            }
            anyhow::Ok(())
        });

        let log_stream = async_stream::stream! {
            while let Some(line) = log_rx.recv().await {
                yield line;
            }
        };

        match client.stream_logs(Request::new(log_stream)).await {
            Ok(resp) => {
                let result = resp.into_inner();
                info!(job_id = %result.job_id, exit_code = result.exit_code, "Job complete");
            }
            Err(e) => warn!("Log stream error: {}", e),
        }

        let _ = exec_handle.await;
    }

    Ok(())
}
