use crate::error::PipelineError;
use crate::pipeline::model::{Pipeline, Stage, Step};

/// Anything that can validate itself
pub trait Validator {
    fn validate(&self) -> Result<(), PipelineError>;
}

impl Validator for Step {
    fn validate(&self) -> Result<(), PipelineError> {
        if self.name.trim().is_empty() {
            return Err(PipelineError::Validation {
                field: "step.name".into(),
                message: "step name cannot be empty".into(),
            });
        }
        if self.run.trim().is_empty() {
            return Err(PipelineError::Validation {
                field: "step.run".into(),
                message: "step must have a 'run' command".into(),
            });
        }
        Ok(())
    }
}

impl Validator for Stage {
    fn validate(&self) -> Result<(), PipelineError> {
        if self.runs_on.trim().is_empty() {
            return Err(PipelineError::Validation {
                field: "stage.runs-on".into(),
                message: "'runs-on' image cannot be empty".into(),
            });
        }
        if self.steps.is_empty() {
            return Err(PipelineError::Validation {
                field: "stage.steps".into(),
                message: "stage must have at least one step".into(),
            });
        }
        for step in &self.steps {
            step.validate()?;
        }
        Ok(())
    }
}

impl Validator for Pipeline {
    fn validate(&self) -> Result<(), PipelineError> {
        if self.name.trim().is_empty() {
            return Err(PipelineError::Validation {
                field: "pipeline.name".into(),
                message: "pipeline name cannot be empty".into(),
            });
        }
        if self.stages.is_empty() {
            return Err(PipelineError::Validation {
                field: "pipeline.stages".into(),
                message: "pipeline must have at least one stage".into(),
            });
        }
        for (stage_name, stage) in &self.stages {
            stage.validate().map_err(|e| match e {
                PipelineError::Validation { field, message } => PipelineError::Validation {
                    field: format!("stages.{}.{}", stage_name, field),
                    message,
                },
                other => other,
            })?;
            // Check that all `needs` references point to existing stages
            if let Some(needs) = &stage.needs {
                for dep in needs {
                    if !self.stages.contains_key(dep.as_str()) {
                        return Err(PipelineError::Validation {
                            field: format!("stages.{}.needs", stage_name),
                            message: format!("unknown dependency '{}'", dep),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
