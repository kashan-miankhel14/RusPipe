/// Integration tests for RustPipe core functionality.
/// Run with: cargo test
use std::collections::HashMap;
use std::fs;

// ── helpers ──────────────────────────────────────────────────────────────────

fn sample_yaml() -> &'static str {
    r#"
name: test-pipeline
stages:
  lint:
    runs-on: rust:latest
    steps:
      - name: Check format
        run: echo "lint ok"
  test:
    runs-on: rust:latest
    needs: [lint]
    steps:
      - name: Run tests
        run: echo "tests ok"
  build:
    runs-on: rust:latest
    needs: [test]
    steps:
      - name: Build
        run: echo "build ok"
"#
}

fn sample_dsl() -> &'static str {
    r#"pipeline my-dsl-pipeline

stage lint
  runs-on rust:latest
  step "Run clippy"
    run cargo clippy
  end
end

stage test
  runs-on rust:latest
  needs lint
  step "Run tests"
    run cargo test
  end
end
"#
}

// ── 1. YAML parsing ───────────────────────────────────────────────────────────

#[test]
fn test_yaml_parse_pipeline_name() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    assert_eq!(p.name, "test-pipeline");
}

#[test]
fn test_yaml_parse_stage_count() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    assert_eq!(p.stages.len(), 3);
}

#[test]
fn test_yaml_parse_needs() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    let test_stage = p.stages.get("test").unwrap();
    assert_eq!(test_stage.needs.as_deref().unwrap(), &["lint"]);
}

// ── 2. DSL parsing ────────────────────────────────────────────────────────────

#[test]
fn test_dsl_parse_pipeline_name() {
    let p = rustpipe::pipeline::dsl::parse_dsl(sample_dsl()).unwrap();
    assert_eq!(p.name, "my-dsl-pipeline");
}

#[test]
fn test_dsl_parse_stage_count() {
    let p = rustpipe::pipeline::dsl::parse_dsl(sample_dsl()).unwrap();
    assert_eq!(p.stages.len(), 2);
}

#[test]
fn test_dsl_parse_needs() {
    let p = rustpipe::pipeline::dsl::parse_dsl(sample_dsl()).unwrap();
    let test_stage = p.stages.get("test").unwrap();
    assert_eq!(test_stage.needs.as_deref().unwrap(), &["lint"]);
}

// ── 3. Validation ─────────────────────────────────────────────────────────────

use rustpipe::pipeline::validator::Validator;

#[test]
fn test_valid_pipeline_passes() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    assert!(p.validate().is_ok());
}

#[test]
fn test_empty_pipeline_name_fails() {
    let yaml = r#"
name: ""
stages:
  lint:
    runs-on: rust:latest
    steps:
      - name: step
        run: echo hi
"#;
    let p: rustpipe::pipeline::model::Pipeline = serde_yaml::from_str(yaml).unwrap();
    assert!(p.validate().is_err());
}

#[test]
fn test_unknown_dependency_fails() {
    let yaml = r#"
name: bad-pipeline
stages:
  test:
    runs-on: rust:latest
    needs: [nonexistent]
    steps:
      - name: step
        run: echo hi
"#;
    let p: rustpipe::pipeline::model::Pipeline = serde_yaml::from_str(yaml).unwrap();
    assert!(p.validate().is_err());
}

// ── 4. DAG wave scheduling ────────────────────────────────────────────────────

use rustpipe::pipeline::dag::build_execution_waves;

#[test]
fn test_dag_wave_count() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    let waves = build_execution_waves(&p).unwrap();
    // lint(wave 0) → test(wave 1) → build(wave 2)
    assert_eq!(waves.len(), 3);
}

#[test]
fn test_dag_lint_in_first_wave() {
    let p: rustpipe::pipeline::model::Pipeline =
        serde_yaml::from_str(sample_yaml()).unwrap();
    let waves = build_execution_waves(&p).unwrap();
    assert!(waves[0].contains(&"lint".to_string()));
}

#[test]
fn test_dag_cycle_detection() {
    let yaml = r#"
name: cyclic
stages:
  a:
    runs-on: ubuntu
    needs: [b]
    steps: []
  b:
    runs-on: ubuntu
    needs: [a]
    steps: []
"#;
    let p: rustpipe::pipeline::model::Pipeline = serde_yaml::from_str(yaml).unwrap();
    assert!(build_execution_waves(&p).is_err());
}

// ── 5. Matrix expand ──────────────────────────────────────────────────────────

use rustpipe::runner::matrix::expand;

#[test]
fn test_matrix_expand_count() {
    let mut m = HashMap::new();
    m.insert("rust".to_string(), vec!["stable".to_string(), "beta".to_string()]);
    m.insert("os".to_string(), vec!["linux".to_string(), "macos".to_string()]);
    let combos = expand(&m);
    assert_eq!(combos.len(), 4);
}

#[test]
fn test_matrix_expand_single_axis() {
    let mut m = HashMap::new();
    m.insert("rust".to_string(), vec!["stable".to_string(), "beta".to_string(), "nightly".to_string()]);
    let combos = expand(&m);
    assert_eq!(combos.len(), 3);
}

#[test]
fn test_matrix_expand_contains_keys() {
    let mut m = HashMap::new();
    m.insert("os".to_string(), vec!["linux".to_string()]);
    let combos = expand(&m);
    assert!(combos[0].contains_key("os"));
    assert_eq!(combos[0]["os"], "linux");
}

// ── 6. Secrets ────────────────────────────────────────────────────────────────

use rustpipe::secrets;

#[test]
fn test_secret_mask_replaces_value() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("token".to_string(), "supersecret".to_string());
    let masked = secrets::mask("echo supersecret", &secrets_map);
    assert_eq!(masked, "echo ***");
}

#[test]
fn test_secret_mask_empty_value_skipped() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("empty".to_string(), "".to_string());
    let masked = secrets::mask("echo hello", &secrets_map);
    assert_eq!(masked, "echo hello");
}

#[test]
fn test_hardcoded_secret_detection() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("token".to_string(), "ghp_abc123".to_string());
    let hits = secrets::check_hardcoded("curl -H 'Authorization: ghp_abc123'", &secrets_map);
    assert!(hits.contains(&"token".to_string()));
}

#[test]
fn test_no_hardcoded_secret_clean() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("token".to_string(), "ghp_abc123".to_string());
    let hits = secrets::check_hardcoded("curl -H 'Authorization: $SECRET_TOKEN'", &secrets_map);
    assert!(hits.is_empty());
}

// ── 7. Cache ──────────────────────────────────────────────────────────────────

use rustpipe::cache;

#[test]
fn test_cache_hash_deterministic() {
    let h1 = cache::stage_hash("lint", &["cargo clippy"], &[]);
    let h2 = cache::stage_hash("lint", &["cargo clippy"], &[]);
    assert_eq!(h1, h2);
}

#[test]
fn test_cache_hash_differs_on_cmd_change() {
    let h1 = cache::stage_hash("lint", &["cargo clippy"], &[]);
    let h2 = cache::stage_hash("lint", &["cargo fmt"], &[]);
    assert_ne!(h1, h2);
}

#[test]
fn test_cache_write_and_check() {
    let hash = cache::stage_hash("test-stage", &["echo hi"], &[]);
    let _ = fs::remove_file(format!(".rustpipe-cache/{}", hash)); // clean up first
    assert!(!cache::check("test-stage", &hash));
    cache::write_cache(&hash).unwrap();
    assert!(cache::check("test-stage", &hash));
    let _ = fs::remove_file(format!(".rustpipe-cache/{}", hash)); // clean up after
}

// ── 8. File-based parse (auto-detect) ────────────────────────────────────────

#[test]
fn test_parse_yaml_file() {
    let path = "/tmp/rustpipe_test.yml";
    fs::write(path, sample_yaml()).unwrap();
    let p = rustpipe::pipeline::parse(path).unwrap();
    assert_eq!(p.name, "test-pipeline");
    let _ = fs::remove_file(path);
}

#[test]
fn test_parse_dsl_file() {
    let path = "/tmp/rustpipe_test.rustpipe";
    fs::write(path, sample_dsl()).unwrap();
    let p = rustpipe::pipeline::parse(path).unwrap();
    assert_eq!(p.name, "my-dsl-pipeline");
    let _ = fs::remove_file(path);
}
