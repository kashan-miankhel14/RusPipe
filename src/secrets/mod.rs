/// M9: Secret masking — redact secret values from log output.
/// Secrets are loaded from env vars prefixed with RUSTPIPE_SECRET_.
use std::collections::HashMap;

/// Load secrets from environment: RUSTPIPE_SECRET_<NAME>=<value>
pub fn load_from_env() -> HashMap<String, String> {
    std::env::vars()
        .filter(|(k, _)| k.starts_with("RUSTPIPE_SECRET_"))
        .map(|(k, v)| {
            let name = k.trim_start_matches("RUSTPIPE_SECRET_").to_lowercase();
            (name, v)
        })
        .collect()
}

/// Replace all secret values in `text` with `***`.
#[allow(dead_code)]
pub fn mask(text: &str, secrets: &HashMap<String, String>) -> String {
    let mut out = text.to_string();
    for val in secrets.values() {
        if !val.is_empty() {
            out = out.replace(val.as_str(), "***");
        }
    }
    out
}

/// Warn if any secret value appears literally in a step's `run:` command.
pub fn check_hardcoded(step_cmd: &str, secrets: &HashMap<String, String>) -> Vec<String> {
    secrets
        .iter()
        .filter(|(_, v)| !v.is_empty() && step_cmd.contains(v.as_str()))
        .map(|(k, _)| k.clone())
        .collect()
}

/// Build env var pairs to inject into a container stage (name → value).
/// Returns (KEY, VALUE) pairs — never log the values.
#[allow(dead_code)]
pub fn env_pairs(secrets: &HashMap<String, String>) -> Vec<(String, String)> {
    secrets
        .iter()
        .map(|(k, v)| (format!("SECRET_{}", k.to_uppercase()), v.clone()))
        .collect()
}
