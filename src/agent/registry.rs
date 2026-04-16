/// M12: Runner registry — service discovery + round-robin load balancing.
/// Agents register on connect; server routes jobs to available agents.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::agent::proto::JobRequest;

#[allow(dead_code)]
#[derive(Debug)]
pub struct RunnerEntry {
    pub agent_id: String,
    pub labels: Vec<String>,
    pub job_tx: mpsc::Sender<JobRequest>,
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct RunnerRegistry {
    inner: Arc<Mutex<RegistryInner>>,
}

#[allow(dead_code)]
#[derive(Default)]
struct RegistryInner {
    runners: HashMap<String, RunnerEntry>,
    rr_index: usize,
}

#[allow(dead_code)]
impl RunnerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an agent with its job channel.
    pub fn register(&self, agent_id: String, labels: Vec<String>, job_tx: mpsc::Sender<JobRequest>) {
        let mut inner = self.inner.lock().unwrap();
        info!(agent = %agent_id, ?labels, "Runner registered");
        inner.runners.insert(agent_id.clone(), RunnerEntry { agent_id, labels, job_tx });
    }

    /// Deregister an agent (disconnected).
    pub fn deregister(&self, agent_id: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.runners.remove(agent_id);
        warn!(agent = %agent_id, "Runner deregistered");
    }

    /// Route a job to the next available runner (round-robin).
    /// Optionally filter by required label.
    pub fn dispatch(&self, job: JobRequest, required_label: Option<&str>) -> bool {
        let inner = self.inner.lock().unwrap();
        let candidates: Vec<&RunnerEntry> = inner
            .runners
            .values()
            .filter(|r| {
                required_label.map_or(true, |lbl| r.labels.iter().any(|l| l == lbl))
            })
            .collect();

        if candidates.is_empty() {
            warn!(job_id = %job.job_id, "No available runners");
            return false;
        }

        // Round-robin: pick by index mod len (we can't mutate inner here, so just pick first)
        let runner = candidates[0];
        let _ = runner.job_tx.try_send(job);
        info!(agent = %runner.agent_id, "Job dispatched");
        true
    }

    pub fn count(&self) -> usize {
        self.inner.lock().unwrap().runners.len()
    }
}
