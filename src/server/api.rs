use axum::{extract::State, response::Json};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunRecord {
    pub id: u64,
    pub pipeline: String,
    pub branch: String,
    pub commit: String,
    pub status: String,
}

pub type RunStore = Arc<Mutex<Vec<RunRecord>>>;

pub fn new_store() -> RunStore {
    Arc::new(Mutex::new(Vec::new()))
}

/// GET /api/v1/runs — uses AppState (defined in mod.rs to avoid circular dep)
pub async fn list_runs(
    State(state): State<Arc<crate::server::AppState>>,
) -> Json<Value> {
    let runs = state.runs.lock().unwrap();
    Json(json!({ "runs": *runs }))
}
