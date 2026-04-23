use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;
use thiserror::Error;

use crate::NickelValue;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvalError {
    #[error("Nickel evaluation failed: {0}")]
    NickelError(String),
    #[error("evaluation timed out")]
    Timeout,
    #[error("import denied")]
    ImportDenied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalOptions {
    pub timeout: Option<Duration>,
    pub import_paths: Vec<PathBuf>,
}

pub trait NickelEvaluator {
    fn evaluate_file(&mut self, path: &Path, options: &EvalOptions) -> Result<NickelValue, EvalError>;
    fn merge(&mut self, left: &NickelValue, right: &NickelValue, options: &EvalOptions)
        -> Result<NickelValue, EvalError>;
    fn call(&mut self, func: &NickelValue, arg: &NickelValue, options: &EvalOptions)
        -> Result<NickelValue, EvalError>;
    fn get_field(
        &mut self,
        value: &NickelValue,
        field_name: &str,
        options: &EvalOptions,
    ) -> Result<NickelValue, EvalError>;
    fn is_function(&mut self, value: &NickelValue, options: &EvalOptions) -> Result<bool, EvalError>;
    fn to_json(&mut self, value: &NickelValue, options: &EvalOptions) -> Result<Value, EvalError>;
}
