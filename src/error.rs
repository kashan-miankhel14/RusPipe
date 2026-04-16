use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse YAML: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Validation error in '{field}': {message}")]
    Validation { field: String, message: String },
}
