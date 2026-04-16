pub mod dag;
pub mod dsl;
pub mod model;
pub mod validator;

use crate::error::PipelineError;
use model::Pipeline;
use sha2::{Digest, Sha256};
use std::fs;

/// Parse a pipeline file — auto-detects YAML (.yml/.yaml) vs DSL (.rustpipe).
pub fn parse(path: &str) -> Result<Pipeline, PipelineError> {
    let contents = fs::read_to_string(path).map_err(|e| PipelineError::FileRead {
        path: path.to_string(),
        source: e,
    })?;

    if path.ends_with(".rustpipe") {
        dsl::parse_dsl(&contents).map_err(|e| PipelineError::Validation {
            field: "dsl".into(),
            message: e.to_string(),
        })
    } else {
        let pipeline: Pipeline = serde_yaml::from_str(&contents)?;
        Ok(pipeline)
    }
}

/// Hash the pipeline file contents for drift detection.
pub fn file_hash(path: &str) -> Option<String> {
    let contents = fs::read(path).ok()?;
    let hash = Sha256::digest(&contents);
    Some(format!("{:x}", hash))
}
