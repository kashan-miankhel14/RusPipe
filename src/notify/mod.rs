/// M10: Notifications (Slack, Discord, generic webhook) + retry with exponential backoff.
use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, warn};

pub struct RunSummary<'a> {
    pub pipeline: &'a str,
    pub branch: &'a str,
    pub commit: &'a str,
    pub status: &'a str,          // "passed" | "failed"
    pub duration_secs: u64,
    pub failed_stage: Option<&'a str>,
}

/// Retry an async operation with exponential backoff.
/// `attempts` = max tries, base delay doubles each time.
async fn with_retry<F, Fut>(attempts: u32, mut f: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut delay = Duration::from_millis(500);
    for attempt in 1..=attempts {
        match f().await {
            Ok(_) => return Ok(()),
            Err(e) if attempt < attempts => {
                warn!(attempt, ?delay, error = %e, "Notification failed, retrying");
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Send a Slack message via incoming webhook URL.
pub async fn notify_slack(webhook_url: &str, summary: &RunSummary<'_>) -> Result<()> {
    let emoji = if summary.status == "passed" { "✅" } else { "❌" };
    let text = format!(
        "{} *{}* — `{}` on `{}` @ `{}` ({}s){}",
        emoji,
        summary.pipeline,
        summary.status,
        summary.branch,
        &summary.commit[..8.min(summary.commit.len())],
        summary.duration_secs,
        summary.failed_stage.map(|s| format!(" — failed at `{}`", s)).unwrap_or_default(),
    );
    let body = json!({ "text": text });
    let client = Client::new();
    with_retry(3, || {
        let client = client.clone();
        let url = webhook_url.to_string();
        let b = body.clone();
        async move {
            client.post(&url).json(&b).send().await?.error_for_status()?;
            Ok(())
        }
    })
    .await?;
    info!(channel = "slack", status = summary.status, "Notification sent");
    Ok(())
}

/// Send a Discord embed via webhook URL.
pub async fn notify_discord(webhook_url: &str, summary: &RunSummary<'_>) -> Result<()> {
    let color: u32 = if summary.status == "passed" { 0x3fb950 } else { 0xf85149 };
    let body = json!({
        "embeds": [{
            "title": format!("{} — {}", summary.pipeline, summary.status.to_uppercase()),
            "color": color,
            "fields": [
                { "name": "Branch",   "value": summary.branch,                                          "inline": true },
                { "name": "Commit",   "value": &summary.commit[..8.min(summary.commit.len())],          "inline": true },
                { "name": "Duration", "value": format!("{}s", summary.duration_secs),                   "inline": true },
            ]
        }]
    });
    let client = Client::new();
    with_retry(3, || {
        let client = client.clone();
        let url = webhook_url.to_string();
        let b = body.clone();
        async move {
            client.post(&url).json(&b).send().await?.error_for_status()?;
            Ok(())
        }
    })
    .await?;
    info!(channel = "discord", status = summary.status, "Notification sent");
    Ok(())
}

/// Generic webhook — POST JSON payload to any URL.
pub async fn notify_webhook(url: &str, summary: &RunSummary<'_>) -> Result<()> {
    let body: Value = json!({
        "pipeline":     summary.pipeline,
        "branch":       summary.branch,
        "commit":       summary.commit,
        "status":       summary.status,
        "duration_secs": summary.duration_secs,
        "failed_stage": summary.failed_stage,
    });
    let client = Client::new();
    with_retry(3, || {
        let client = client.clone();
        let u = url.to_string();
        let b = body.clone();
        async move {
            client.post(&u).json(&b).send().await?.error_for_status()?;
            Ok(())
        }
    })
    .await?;
    info!(channel = "webhook", url = %url, status = summary.status, "Notification sent");
    Ok(())
}
