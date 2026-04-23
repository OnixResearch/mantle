use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
pub enum SystemConfigError {
    #[error("cli error: {message}")]
    Cli {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("loader error: {message}")]
    Loader {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("inventory error: {message}")]
    Inventory {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("cross-reference error: {message}")]
    CrossRef {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("evaluation error: {message}")]
    Eval {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("fragment error: {message}")]
    Fragment {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
    #[error("assembler error: {message}")]
    Assembler {
        message: String,
        detail: Option<String>,
        machine_name: Option<String>,
        module_name: Option<String>,
        field_path: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
pub enum SystemConfigWarning {
    #[error("orphan provider consumption: {provider_type}")]
    OrphanProviderConsumption {
        provider_type: String,
        machine_name: Option<String>,
        module_name: Option<String>,
    },
}
