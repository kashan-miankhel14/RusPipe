/// GitHub API client using hyper directly (low-level HTTP — M5 requirement).
/// Used to post commit status checks back to GitHub.
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde_json::{json, Value};
use tracing::{info, warn};

/// Post a commit status to GitHub via the low-level hyper client.
/// `state` is one of: "pending", "success", "failure", "error"
#[allow(dead_code)]
pub async fn post_commit_status(
    token: &str,
    repo: &str, // "owner/repo"
    sha: &str,
    state: &str,
    description: &str,
) -> anyhow::Result<()> {
    let body_json = json!({
        "state": state,
        "description": description,
        "context": "rustpipe/ci"
    });
    let body_bytes = serde_json::to_vec(&body_json)?;

    let uri = format!(
        "https://api.github.com/repos/{}/statuses/{}",
        repo, sha
    );

    let req = Request::post(&uri)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("User-Agent", "rustpipe/0.1")
        .body(Full::new(Bytes::from(body_bytes)))?;

    // hyper low-level HTTPS client
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_only()
        .enable_http1()
        .build();
    let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(https);

    let resp: Response<_> = client.request(req).await?;
    let status = resp.status();

    if status.is_success() {
        info!(repo = %repo, sha = %&sha[..8.min(sha.len())], state = %state, "GitHub status posted");
    } else {
        let body_bytes = resp.into_body().collect().await?.to_bytes();
        let msg = String::from_utf8_lossy(&body_bytes);
        warn!(http_status = %status, body = %msg, "GitHub status post failed");
    }

    Ok(())
}

/// Parse a GitHub push event payload into (branch, sha, repo).
#[allow(dead_code)]
pub fn parse_push_event(payload: &Value) -> (String, String, String) {
    let branch = payload["ref"]
        .as_str()
        .unwrap_or("unknown")
        .trim_start_matches("refs/heads/")
        .to_string();
    let sha = payload["after"].as_str().unwrap_or("unknown").to_string();
    let repo = payload["repository"]["full_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    (branch, sha, repo)
}

/// M14: Full GitOps loop helper — wraps pipeline lifecycle with GitHub status checks.
/// Call start() on run begin, stage_done() per stage, finish() on completion.
#[allow(dead_code)]
pub struct GitOpsLoop<'a> {
    pub token: &'a str,
    pub repo: &'a str,
    pub sha: &'a str,
    pub dashboard_url: String,
}

#[allow(dead_code)]
impl<'a> GitOpsLoop<'a> {
    pub fn new(token: &'a str, repo: &'a str, sha: &'a str, port: u16) -> Self {
        Self {
            token,
            repo,
            sha,
            dashboard_url: format!("https://localhost:{}/dashboard", port),
        }
    }

    pub async fn start(&self) {
        let _ = post_commit_status(
            self.token, self.repo, self.sha,
            "pending", "RustPipe: pipeline started",
        ).await;
    }

    pub async fn stage_done(&self, stage: &str, passed: bool) {
        let state = if passed { "success" } else { "failure" };
        let desc = format!("RustPipe: stage '{}' {}", stage, if passed { "passed" } else { "failed" });
        let _ = post_commit_status(self.token, self.repo, self.sha, state, &desc).await;
    }

    pub async fn finish(&self, passed: bool) {
        let state = if passed { "success" } else { "failure" };
        let desc = format!("RustPipe: pipeline {} — {}", state, self.dashboard_url);
        let _ = post_commit_status(self.token, self.repo, self.sha, state, &desc).await;
    }
}
