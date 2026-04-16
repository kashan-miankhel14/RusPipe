use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub attempts: u32,
    pub backoff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub run: String,
    pub artifact: Option<String>,
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    #[serde(rename = "runs-on")]
    pub runs_on: String,
    pub steps: Vec<Step>,
    pub needs: Option<Vec<String>>,
    pub when: Option<String>,
    pub timeout_secs: Option<u64>,
    /// matrix: { rust: [stable, beta], os: [linux, macos] }
    pub matrix: Option<HashMap<String, Vec<String>>>,
    #[serde(rename = "fail-fast", default)]
    pub fail_fast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub branches: Option<Vec<String>>,
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyConfig {
    pub slack: Option<String>,
    pub discord: Option<String>,
    pub webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub trigger: Option<Trigger>,
    pub stages: HashMap<String, Stage>,
    pub secrets: Option<Vec<String>>,
    pub notify: Option<NotifyConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageStatus {
    #[allow(dead_code)]
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

impl std::fmt::Display for StageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageStatus::Pending  => write!(f, "PENDING"),
            StageStatus::Running  => write!(f, "RUNNING"),
            StageStatus::Passed   => write!(f, "PASSED"),
            StageStatus::Failed   => write!(f, "FAILED"),
            StageStatus::Skipped  => write!(f, "SKIPPED"),
        }
    }
}
